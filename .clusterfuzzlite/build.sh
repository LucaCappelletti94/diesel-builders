#!/bin/bash
set -eu
export CARGO_BUILD_JOBS=2

cd "$SRC/diesel-builders"
cargo fuzz build -O --debug-assertions --fuzz-dir fuzz

expected="table_model relationship_attributes index"
targets=$(cargo fuzz list --fuzz-dir fuzz)
actual=$(printf '%s\n' $targets | LC_ALL=C sort)
want=$(printf '%s\n' $expected | LC_ALL=C sort)
if [[ "$actual" != "$want" ]]; then
    echo "fuzz targets are [$targets], expected [$expected]" >&2
    exit 1
fi

target_dir=fuzz/target/x86_64-unknown-linux-gnu/release
for name in $expected; do
    cp "$target_dir/$name" "$OUT/"

    options_file=fuzz/seeds/$name/$name.options
    if [[ ! -f "$options_file" ]]; then
        echo "fuzz target $name is missing $options_file" >&2
        exit 1
    fi
    cp "$options_file" "$OUT/$name.options"

    files=()
    while IFS= read -r -d '' file; do
        files+=("$file")
    done < <(find "fuzz/seeds/$name" -maxdepth 1 -type f ! -name "$name.options" -print0)
    if [[ ${#files[@]} -eq 0 ]]; then
        echo "fuzz target $name has no seed corpus" >&2
        exit 1
    fi
    zip -qj "$OUT/${name}_seed_corpus.zip" "${files[@]}"
done
