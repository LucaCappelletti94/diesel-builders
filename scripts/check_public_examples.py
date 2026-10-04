#!/usr/bin/env python3
"""Gate owned public methods for executable doc examples.

Scans nightly rustdoc JSON documents (numeric root id, string-keyed item
index) and requires every in-scope method to carry at least one executable
Rust code fence.

In scope are all methods of reachable, local, public traits and all public
inherent methods of reachable local public structs, enums, and unions.
Reachable means the item sits inside a public module chain from the crate
root or is exposed by a public reexport, including glob reexports. External
items, foreign trait impls, and items in private modules that no public
reexport exposes are out of scope. Methods are deduplicated by item id.
Usage:
    check_public_examples.py FILE...
    check_public_examples.py DIR...

Files must be rustdoc JSON documents. Directories contribute every *.json
member that parses as a rustdoc document; other JSON members are skipped
with a note. The gate exits 1 when zero methods were checked across all
documents, or when any checked method lacks an executable example.
"""

import json
import os
import re
import sys

NON_EXECUTABLE = frozenset({"ignore", "no_run", "compile_fail"})
# rustdoc doctest tags that do not change whether the block is Rust.
NEUTRAL_TAGS = frozenset({"should_panic", "test_harness", "standalone_crate"})
EDITION_RE = re.compile(r"^edition")
FENCE_RE = re.compile(r"^\s{0,3}(`{3,}|~{3,})\s*(.*)$")


def fence_info_is_executable(info):
    """Report whether a fence info string names an executable Rust block.

    Mirrors rustdoc's LangString semantics: tags are matched case-sensitively;
    a block is Rust when it carries the ``rust`` tag or no unknown tag at
    all, and it executes only when it carries none of ``ignore`` (including
    the platform-specific ``ignore-`` form), ``no_run``, or ``compile_fail``,
    wherever those tags appear in the info string.
    """
    has_rust_tag = False
    has_unknown_tag = False
    for raw in re.split(r"[,\s]+", info.strip()):
        if not raw:
            continue
        if raw in NON_EXECUTABLE or raw.startswith("ignore-"):
            return False
        if raw == "rust":
            has_rust_tag = True
        elif raw == "custom":
            return False
        elif raw in NEUTRAL_TAGS or EDITION_RE.match(raw):
            continue
        else:
            has_unknown_tag = True
    return has_rust_tag or not has_unknown_tag


TYPE_KINDS = frozenset({"struct", "enum", "union"})


def fence_infos(docs):
    """Yield the info string of every code fence, closed or open to EOF."""
    if not docs:
        return
    fence = None
    for line in docs.splitlines():
        match = FENCE_RE.match(line)
        if match is None:
            continue
        if fence is None:
            fence = (match.group(1)[0], len(match.group(1)))
            yield match.group(2).strip()
        elif (
            match.group(1)[0] == fence[0]
            and len(match.group(1)) >= fence[1]
            and not match.group(2).strip()
        ):
            fence = None


def has_executable_example(docs):
    """Report whether the docs carry at least one executable Rust fence."""
    return any(fence_info_is_executable(info) for info in fence_infos(docs))


def kind_of(item):
    """Return the inner kind of an item, or None when it has no inner."""
    inner = item.get("inner")
    if isinstance(inner, dict) and len(inner) == 1:
        return next(iter(inner))
    return None


def load_document(path):
    """Load a rustdoc JSON document; raise ValueError when it is not one."""
    with open(path, encoding="utf-8") as fh:
        data = json.load(fh)
    if not isinstance(data, dict):
        raise ValueError("top level is not an object")
    root = data.get("root")
    index = data.get("index")
    if not isinstance(root, int) or not isinstance(index, dict):
        raise ValueError("missing numeric root or string-keyed index")
    root_item = index.get(str(root))
    if not isinstance(root_item, dict) or not isinstance(root_item.get("inner"), dict):
        raise ValueError("root item is not in the index")
    return data


def check_crate(data):
    """Return (checked, missing) for one rustdoc document."""
    index = data["index"]
    paths = data.get("paths") or {}
    root = data["root"]
    root_item = index[str(root)]
    base = (root_item.get("name") or "crate",)

    def method_name(mid, path):
        summary = paths.get(str(mid))
        if isinstance(summary, dict) and summary.get("path"):
            return "::".join(summary["path"])
        item = index[str(mid)]
        return "::".join(path + (item.get("name") or "?",))

    def is_method(mid):
        item = index.get(str(mid))
        return (
            isinstance(item, dict)
            and item.get("crate_id") == 0
            and kind_of(item) == "function"
        )

    seen = set()
    missing = []

    def record(mid, path):
        if mid in seen:
            return
        seen.add(mid)
        docs = index[str(mid)].get("docs")
        if not has_executable_example(docs):
            missing.append(method_name(mid, path))

    queue = [(root, "public", base)]
    while queue:
        item_id, how, path = queue.pop()
        item = index.get(str(item_id))
        if not isinstance(item, dict) or item.get("crate_id") != 0:
            continue
        if how == "public" and item.get("visibility") != "public":
            continue
        kind = kind_of(item)
        if kind is None:
            continue
        payload = next(iter(item["inner"].values()))

        if kind == "module":
            child_path = path if payload.get("is_crate") else path + (item.get("name") or "?",)
            for child in payload.get("items", []):
                queue.append((child, "public", child_path))
        elif kind == "use":
            target = payload.get("id")
            if payload.get("is_glob"):
                target_item = index.get(str(target))
                if (
                    not isinstance(target_item, dict)
                    or target_item.get("crate_id") != 0
                    or kind_of(target_item) != "module"
                ):
                    continue
                # A glob reexport exposes exactly the public children of the
                # module, directly at the current path.
                for child in target_item["inner"]["module"].get("items", []):
                    queue.append((child, "public", path))
            else:
                name = payload.get("name")
                if not name:
                    target_item = index.get(str(target))
                    name = target_item.get("name") if isinstance(target_item, dict) else None
                # A public reexport exposes the target at this path even
                # when the target itself is not public.
                queue.append((target, "reexport", path + (name or "?",)))
        elif kind == "trait":
            for mid in payload.get("items", []):
                if is_method(mid):
                    record(mid, path)
        elif kind in TYPE_KINDS:
            for impl_id in payload.get("impls", []):
                impl = index.get(str(impl_id))
                if not isinstance(impl, dict) or impl.get("crate_id") != 0:
                    continue
                if kind_of(impl) != "impl":
                    continue
                spec = impl["inner"]["impl"]
                if spec.get("trait") is not None:
                    # Foreign trait impls are out of scope.
                    continue
                for mid in spec.get("items", []):
                    if not is_method(mid):
                        continue
                    if index[str(mid)].get("visibility") != "public":
                        continue
                    record(mid, path)

    return len(seen), missing


def main(argv):
    if len(argv) < 2:
        print("usage: check_public_examples.py FILE_OR_DIR...", file=sys.stderr)
        return 2
    explicit = set()
    files = []
    for arg in argv[1:]:
        if os.path.isfile(arg):
            explicit.add(arg)
            files.append(arg)
        elif os.path.isdir(arg):
            files.extend(
                os.path.join(arg, name)
                for name in sorted(os.listdir(arg))
                if name.endswith(".json")
            )
        else:
            print("%s: no such file or directory" % arg, file=sys.stderr)
            return 2
    if not files:
        print("no rustdoc JSON files given", file=sys.stderr)
        return 2

    total_checked = 0
    total_missing = 0
    documents = 0
    for path in files:
        try:
            data = load_document(path)
        except OSError as exc:
            print("%s: cannot read: %s" % (path, exc), file=sys.stderr)
            return 2
        except ValueError as exc:
            if path in explicit:
                print("%s: not a rustdoc JSON document: %s" % (path, exc), file=sys.stderr)
                return 2
            print("skipping %s: not a rustdoc JSON document (%s)" % (path, exc), file=sys.stderr)
            continue
        checked, missing = check_crate(data)
        documents += 1
        crate_name = data["index"][str(data["root"])].get("name") or os.path.basename(path)
        if missing:
            print("%s: checked %d in-scope methods, %d missing an executable example"
                  % (crate_name, checked, len(missing)))
            for name in missing:
                print("  %s" % name)
        elif checked == 0:
            print("%s: checked 0 in-scope methods" % crate_name)
        else:
            print("%s: checked %d in-scope methods, all with executable examples"
                  % (crate_name, checked))
        total_checked += checked
        total_missing += len(missing)

    if documents == 0:
        print("no rustdoc JSON documents found", file=sys.stderr)
        return 2
    if total_checked == 0:
        print("checked 0 methods in %d document(s); refusing to pass" % documents,
              file=sys.stderr)
        return 1
    print("checked %d in-scope methods in %d document(s), %d missing an executable example"
          % (total_checked, documents, total_missing))
    return 1 if total_missing else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
