# Diesel registry release prerequisite

Publication of `diesel-builders` requires a compatible registry release of `diesel`. The latest published version is `2.3.13`. Registry compilation fails on the missing `diesel::query_source::QueryRelation` and `HasTable` support for `Option<T>` required by nested model loading.

## Reproduction

```sh
cargo search diesel --limit 1
nice -n 10 env RUSTC_WRAPPER=sccache cargo package -p diesel-builders-derive -p diesel-builders --allow-dirty -j 2
```

The derive archive verifies. The core archive resolves registry dependencies and fails in `load_nested_query_builder/nested_select.rs` and optional nested-model bounds. Registry `tuplities` also causes independent errors.

## Required release

A compatible release must contain the required APIs available at [`81f1cdb2`](https://github.com/diesel-rs/diesel/commit/81f1cdb2). Recheck generated upsert and composite-join consumers against the release before declaring publication ready. Successful archive generation with `--no-verify` establishes packaging metadata only.

## Prior art and affected branches

The `QueryRelation` tracker search found merged pull requests [#4776](https://github.com/diesel-rs/diesel/pull/4776), which refactors query sources, and [#4961](https://github.com/diesel-rs/diesel/pull/4961), which adds feature-gated columns. These changes already belong to upstream development.

The registry failure is reproduced on `diesel-builders` branch `remove-codacy`. Registry compilation of `main` and `switch-to-upstream-diesel` was not checked in this phase. Git-main dependencies compile with the consumer's declared Rust `1.88` baseline. No upstream changes or tracker posts were made.
