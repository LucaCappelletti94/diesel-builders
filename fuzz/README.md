# Fuzz targets for `diesel-builders-derive`

The targets source-include the derive modules from `diesel-builders-derive` so they exercise the production parser and expansion code directly. `table_model` consumes raw bytes. The other targets consume UTF-8 text and reject non-UTF-8 inputs before the body runs.

## Input encodings

- `table_model`. A byte decision tape generates a `syn::DeriveInput` with at most four recursive type or default-expression layers, 32 fields, eight attributes per container or field, and four entries per relationship list. Choices cover identifiers, generics, primitive and custom types, defaults, primary and foreign keys, inheritance, keyed `same_as`, mandatory and discretionary relations, SQL overrides, and malformed attributes. Tuple structs, unit structs, enums, and unions exercise rejected model shapes.
- `relationship_attributes`. One `#[...]` attribute per line, every line parseable as a `syn::Attribute`. Each attribute is applied to the container and to every field of a fixed two-field template struct before the expansion runs.
- `index`. The token stream inside an `index!(...)` or `unique_index!(...)` invocation. A comma-terminated list of column types with an optional trailing comma. The marker trait is picked from the input length so both `IndexedColumn` and `UniquelyIndexedColumn` are exercised.

`table_model` reads model identity, visibility, body kind, generics, container attributes, and fields in that order. Field choices read identity, visibility, type, and attributes. Exhausted input supplies zero choices, and `fuzz/src/model_input.rs` defines the selectors.

## Invariants

- `table_model`. Every accepted input must expand to a valid `syn::File` that declares the `diesel::table!` definition and a `TableExt` implementation whose `Model` is the input struct. This is the same contract the native derive tests check.
- `relationship_attributes`. The field-level `#[mandatory(...)]`/`#[discretionary(...)]` extraction and the field attribute validation must agree with the full expansion. Invalid or conflicting attributes never expand, and accepted inputs expand to a non-empty valid `syn::File`.
- `index`. Each column at position `i` must produce exactly `impl #trait<::diesel_builders::typenum::U{i}, (#columns, )> for #column`, token for token against the documented shared expansion. Bodies that are not a terminated type list must be rejected.

## Seed corpus

`fuzz/seeds/<target>/` holds the committed starting corpus. One input per file, in that target's encoding, plus a `<target>.options` file that carries the `[libfuzzer]` `max_len` cap. `.clusterfuzzlite/build.sh` zips each directory into `<target>_seed_corpus.zip` and fails the build when a target has no seeds or no options.

Decode a `table_model` seed or crash input into model source with `cargo run --manifest-path fuzz/Cargo.toml --example decode_model -- <input-path>`. The `table_model` byte budget is `max_len = 2048`.

## Local replay

```sh
cargo fuzz build -O --debug-assertions --fuzz-dir fuzz
for name in table_model relationship_attributes index; do
  ./fuzz/target/x86_64-unknown-linux-gnu/release/$name fuzz/seeds/$name -runs=0
done
```

A bounded run of one target, with a scratch corpus directory:

```sh
cargo fuzz run -O --debug-assertions --fuzz-dir fuzz table_model \
  fuzz/corpus/table_model fuzz/seeds/table_model -max_total_time=600 -rss_limit_mb=2560
```

## ClusterFuzzLite wiring

`.clusterfuzzlite/Dockerfile` pins the `base-builder-rust` image by digest and installs `cargo-fuzz 0.13.2`. `build.sh` builds with `CARGO_BUILD_JOBS=2` and copies each binary, options file, and seed zip to `$OUT`. The `cflite_*.yml` workflows run PR and batch fuzzing against the `fuzz-corpus` storage branch, and `cflite_pr.yml` replays the committed seeds deterministically on every pull request.
