#![doc = include_str!("../../README.md")]

// Error handling helpers
pub mod builder_error;
pub use builder_error::{BuilderError, BuilderResult, IncompleteBuilderError};

// Re-exported modules from diesel-additions
pub mod tables;
pub use tables::{NonCompositePrimaryKeyTables, Tables};
pub mod table_model;
pub use table_model::TableModel;
pub mod typed;
pub use typed::Typed;
pub mod typed_column;
pub use typed_column::TypedColumn;
pub mod get_column;
pub use get_column::{GetColumn, GetColumnExt, MayGetColumn, MayGetColumnExt};
pub mod get_set_columns;
pub use get_set_columns::{
    GetColumns, MayGetColumns, MaySetColumns, SetColumns, TryMaySetColumns, TrySetColumns,
    TrySetHomogeneous, TupleGetColumns, TupleMayGetColumns,
};
pub mod columns;
pub use columns::Columns;
pub mod table_addition;
pub use table_addition::{HasTableAddition, TableAddition};
pub mod set_column;
pub use set_column::{MaySetColumn, SetColumn, SetColumnExt, TrySetColumn, TrySetColumnExt};
pub mod insertable_table_model;
pub use insertable_table_model::InsertableTableModel;
pub mod foreign_key;
pub use foreign_key::{
    ForeignKey, HasPrimaryKeyColumn, HostColumn, IndexedColumn, SingletonForeignKey, TableIndex,
};
pub mod flat_insert;
pub use flat_insert::FlatInsert;

// Re-exported modules from diesel-relations
pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as_group;
pub use ancestors::{AncestorOfIndex, Descendant, DescendantOf, Root};
pub use horizontal_same_as::{
    DiscretionarySameAsIndex, HorizontalSameAsColumn, HorizontalSameAsKey, HorizontalSameAsKeys,
    MandatorySameAsIndex,
};
pub mod horizontal_same_as_group;
pub use horizontal_same_as_group::HorizontalSameAsGroup;

// Original diesel-builders modules
pub mod buildable_columns;
pub mod buildable_table;
pub mod buildable_tables;
pub mod table_builder;
pub use buildable_columns::BuildableColumn;
pub use buildable_table::BuildableTable;
pub use buildable_tables::BuildableTables;
pub use table_builder::RecursiveBuilderInsert;
pub use table_builder::TableBuilder;
pub mod set_builder;
pub use set_builder::{
    SetDiscretionaryBuilder, SetDiscretionaryBuilderExt, SetDiscretionaryModel,
    SetDiscretionaryModelExt, SetMandatoryBuilder, SetMandatoryBuilderExt,
    TryMaySetDiscretionarySameAsColumn, TryMaySetDiscretionarySameAsColumns,
    TrySetDiscretionaryBuilder, TrySetDiscretionaryBuilderExt, TrySetDiscretionaryModel,
    TrySetDiscretionaryModelExt, TrySetMandatoryBuilder, TrySetMandatoryBuilderExt,
    TrySetMandatorySameAsColumn, TrySetMandatorySameAsColumns,
};
pub mod get_builder;
pub use get_builder::{GetBuilder, MayGetBuilder};
pub mod nested_insert;
pub use nested_insert::Insert;
pub mod builder_bundle;
pub use builder_bundle::CompletedTableBuilderBundle;
pub use builder_bundle::{BundlableTable, TableBuilderBundle};
pub mod bundlable_tables;
pub use bundlable_tables::BundlableTables;
pub mod get_foreign;
pub use get_foreign::GetForeign;

/// Re-export typenum for convenience
pub mod typenum {
    pub use typenum::*;
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

    // Re-export commonly used macros from diesel_builders_macros
    pub use diesel_builders_macros::{
        Decoupled, GetColumn, HasTable, MayGetColumn, Root, SetColumn, TableModel, descendant_of,
        fk, index,
    };

    pub use crate::insertable_table_model::InsertableTableModel;

    // Table relationship traits
    pub use crate::ancestors::{Descendant, DescendantOf};
    // Core table building traits
    pub use crate::buildable_table::BuildableTable;
    // Column accessor extension traits (always use Ext variants)
    pub use crate::get_column::{GetColumnExt, MayGetColumnExt};
    // Note: Root is NOT exported here to avoid collision with Root macro from
    // diesel_builders_macros
    pub use crate::horizontal_same_as::{HorizontalSameAsKey, HorizontalSameAsKeys};
    // Builder setter extension traits (always use Ext variants)
    pub use crate::set_builder::{
        SetDiscretionaryBuilderExt, SetDiscretionaryModelExt, SetMandatoryBuilderExt,
        TrySetDiscretionaryBuilderExt, TrySetDiscretionaryModelExt, TrySetMandatoryBuilderExt,
    };
    pub use crate::{
        builder_bundle::BundlableTable,
        nested_insert::Insert,
        set_column::{SetColumnExt, TrySetColumnExt},
        table_addition::TableAddition,
    };
}
