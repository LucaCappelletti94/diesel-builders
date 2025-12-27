#![doc = include_str!("../../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

// Error handling helpers
pub mod builder_error;
pub use builder_error::{BuilderError, BuilderResult, IncompleteBuilderError};

// Re-exported modules from diesel-additions
pub mod tables;
pub use tables::{HasNestedTables, NestedTables, Tables};
pub mod table_model;
pub use table_model::TableModel;
pub mod table_models;
pub use table_models::NestedTableModels;
pub mod typed;
pub use typed::*;
pub mod typed_column;
pub use typed_column::TypedColumn;
pub mod get_column;
pub use get_column::{GetColumn, GetColumnExt, MayGetColumn, MayGetColumnExt};
pub mod get_set_columns;
pub use get_set_columns::*;
pub mod columns;
pub use columns::{Columns, NestedColumns};
pub mod table_addition;
pub use table_addition::{HasTableExt, TableExt};
pub mod set_column;
pub use set_column::{
    MaySetColumn, SetColumn, SetColumnExt, TrySetColumn, TrySetColumnExt, ValidateColumn,
};
pub mod foreign_key;
pub use foreign_key::*;

// Re-exported modules from diesel-relations
pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as_group;
pub use ancestors::{
    AncestorOfIndex, Descendant, DescendantOf, ModelDelete, ModelDescendantExt, ModelFind,
    ModelUpsert, Root,
};
pub use horizontal_same_as::*;
pub use vertical_same_as_group::VerticalSameAsGroup;
pub mod horizontal_same_as_group;
pub use horizontal_same_as_group::HorizontalSameAsGroup;

pub mod buildable_table;
pub mod nested_buildable_tables;
pub mod table_builder;
pub use buildable_table::*;
pub use nested_buildable_tables::*;
pub use table_builder::RecursiveBuilderInsert;
pub use table_builder::TableBuilder;
pub mod set_builder;
pub use set_builder::*;
pub mod nested_insert;
pub use nested_insert::Insert;
pub mod builder_bundle;
pub use builder_bundle::{
    BundlableTable, CompletedTableBuilderBundle, RecursiveBundleInsert, TableBuilderBundle,
};
pub mod nested_bundlable_tables;
pub use nested_bundlable_tables::*;
pub mod get_foreign;
pub use get_foreign::{GetForeign, GetForeignExt};
pub mod load_query_builder;
pub use load_query_builder::{LoadFirst, LoadMany, LoadManySorted, LoadQueryBuilder};

/// Re-export typenum for convenience
pub mod typenum {
    pub use typenum::*;
}

/// Re-export tuplities for convenience
pub mod tuplities {
    pub use tuplities::prelude::*;
}

pub mod prelude {
    //! Prelude module containing the most commonly used items from
    //! diesel-builders.
    //!
    //! This module re-exports the most frequently used traits and types, making
    //! it convenient to import everything you need with a single use
    //! statement:
    //!
    //! ```rust
    //! use diesel_builders::prelude::*;
    //! ```

    // Re-export diesel prelude for convenience
    pub use diesel::prelude::*;
    // Table model trait - not exported to avoid collision with TableModel macro
    // pub use crate::table_model::TableModel;

    // Re-export commonly used macros from diesel_builders_derive
    // Note: GetColumn is now automatically implemented by TableModel derive
    pub use diesel_builders_derive::{TableModel, const_validator, fk, fpk, index, unique_index};

    pub use crate::get_foreign::GetForeignExt;

    // Table relationship traits
    pub use crate::ancestors::{
        Descendant, DescendantOf, ModelDescendantExt, ModelFind, ModelUpsert,
    };
    // Core table building traits
    pub use crate::buildable_table::BuildableTable;
    // Column accessor extension traits (always use Ext variants)
    pub use crate::get_column::{GetColumnExt, MayGetColumnExt};
    // Note: Root is NOT exported here to avoid collision with Root macro from
    // diesel_builders_derive
    pub use crate::horizontal_same_as::HorizontalKey;
    // Builder setter extension traits (always use Ext variants)
    /// Query loading traits
    pub use crate::load_query_builder::{LoadFirst, LoadMany, LoadManySorted};
    pub use crate::set_builder::{
        SetDiscretionaryBuilderExt, SetDiscretionaryModelExt, SetMandatoryBuilderExt,
        TrySetDiscretionaryBuilderExt, TrySetDiscretionaryModelExt, TrySetMandatoryBuilderExt,
    };
    pub use crate::{
        builder_bundle::BundlableTable,
        nested_insert::Insert,
        set_column::{SetColumnExt, TrySetColumnExt, ValidateColumn},
        table_addition::TableExt,
    };
}
