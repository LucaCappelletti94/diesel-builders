# Tuple registry release prerequisite

Publication of `diesel-builders` requires a compatible registry release of `tuplities` and its workspace dependencies. The latest published `tuplities` is `0.1.4`. Its `flatten-nest` feature lacks `NestedTupleIntoVec` and `NestedTupleChain`, and its nested-reference contract fails the consumer's `IntoNestedTupleOption` bounds.

## Reproduction

```sh
cargo search tuplities --limit 1
nice -n 10 env RUSTC_WRAPPER=sccache cargo package -p diesel-builders-derive -p diesel-builders --allow-dirty -j 2
```

The derive archive verifies. The core archive resolves registry dependencies and fails with missing `tuplities::prelude::NestedTupleIntoVec`, missing `NestedTupleChain`, and incompatible `NestedTupleRef::Ref` bounds. Registry `diesel` also causes independent errors.

A minimal registry dependency uses the following manifest entry.

```toml
[dependencies]
tuplities = { version = "0.1.4", default-features = false, features = ["flatten-nest"] }
```

```rust
pub use tuplities::prelude::{NestedTupleChain, NestedTupleIntoVec};
```

## Required release

Publish compatible packages containing the APIs available at [`fdb304fc`](https://github.com/LucaCappelletti94/tuplities/commit/fdb304fc). Keep the `flatten-nest` exports and canonical nested-reference contracts usable together. The registry incompatibility above blocks publication until then.

## Prior art and affected branches

Tracker searches for `NestedTupleChain` and `NestedTupleIntoVec` returned no issues or pull requests. The release search returned only unrelated dependency-update pull requests [#1](https://github.com/LucaCappelletti94/tuplities/pull/1) and [#2](https://github.com/LucaCappelletti94/tuplities/pull/2).

The registry failure is reproduced on `diesel-builders` branch `remove-codacy`. Registry compilation of `main` and `switch-to-upstream-diesel` was not checked in this phase. Git-main dependencies compile with the consumer's declared Rust `1.88` baseline. No upstream changes or tracker posts were made.
