# diesel-builders-derive

[![Coverage](https://codecov.io/gh/LucaCappelletti94/diesel-builders/branch/main/graph/badge.svg)](https://codecov.io/gh/LucaCappelletti94) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Docs](https://docs.rs/diesel-builders-derive/badge.svg)](https://docs.rs/diesel-builders-derive) [![Crates.io](https://img.shields.io/crates/v/diesel-builders-derive.svg)](https://crates.io/crates/diesel-builders-derive)

The documentation and registry badges resolve after the first release.

Procedural macros for [`diesel-builders`](https://github.com/LucaCappelletti94/diesel-builders). Use the macros through that crate's re-exports.

`TableModel` generates the Diesel table, checked builders, column traits, and relationship implementations from a model definition. Declare related query groups with `diesel::allow_tables_to_appear_in_same_query!`.

Raw identifiers are supported for model and column names, including columns marked `#[infallible]` or `#[table_model(infallible)]`.

`index!` and `unique_index!` describe indexed column groups for relationship queries. See the [executable model examples](https://github.com/LucaCappelletti94/diesel-builders#examples) for attribute syntax and builder usage.
