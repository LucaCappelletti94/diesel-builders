#![doc = include_str!("../README.md")]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::indexing_slicing)]
#![deny(clippy::allow_attributes)]
#![deny(clippy::allow_attributes_without_reason)]
#![deny(clippy::fallible_impl_from)]

// Error handling helpers
pub mod builder_error;
pub use builder_error::{BuilderError, BuilderResult, IncompleteBuilderError};

// Re-exported modules from diesel-additions
pub mod tables;
pub use tables::{HasNestedTables, NestedTables, Tables};
pub mod table_model;
pub use table_model::TableModel;
pub mod get_model;
pub use get_model::*;
pub mod table_models;
pub use table_models::NestedTableModels;
pub mod typed;
pub use typed::*;
pub mod typed_column;
pub use typed_column::{DynColumn, TypedColumn};
pub mod get_column;
pub use get_column::{
    GetColumn, GetColumnExt, MayGetColumn, MayGetColumnExt, TryGetDynamicColumn,
    TryGetDynamicColumns,
};
pub mod get_set_columns;
pub use get_set_columns::*;
pub mod columns;
pub use columns::{Columns, HasNestedDynColumns, NestedColumns, NestedDynColumns};
pub mod table_addition;
pub use table_addition::{HasTableExt, TableExt};
pub mod set_column;
pub use set_column::{
    MaySetColumn, SetColumn, SetColumnExt, TrySetColumn, TrySetColumnExt, TrySetDynamicColumn,
    ValidateColumn,
};
mod mutation;
mod mutation_same_as;
mod validation;
pub use validation::{
    EitherValidationError, TakeColumn, TryMaySetValues, TrySetHomogeneousValues, TrySetValues,
    ValidateIncomingValues, ValidateOptionalIncomingValues, ValidateRecord, ValidateStoredValues,
};
pub mod construction;
pub use construction::{BuildDefaults, CheckAndMoveColumns, DefaultBuilder, DefaultBundle};
pub mod foreign_key;
pub use foreign_key::*;

// Re-exported modules from diesel-relations
pub mod ancestors;
pub mod horizontal_same_as;
pub mod vertical_same_as_group;
pub use ancestors::{
    AncestorOfIndex, Descendant, DescendantOf, DescendantWithSelf, ModelDelete, ModelDescendantExt,
    ModelFind, ModelUpsert, Root,
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
pub use table_builder::{RecursiveBuilderInsert, TableBuilder};
pub mod set_builder;
pub use set_builder::*;
mod insert_transaction;
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
pub use load_query_builder::{LoadFirst, LoadMany, LoadQueryBuilder, LoadSorted};
pub mod load_nested_query_builder;

/// Module defining helper types for the crate.
pub mod helper_type;
pub use helper_type::*;

/// Re-export typenum for convenience
pub mod typenum {
    pub use typenum::*;
}

/// Re-export tuplities for convenience
pub mod tuplities {
    pub use tuplities::prelude::*;
}

pub mod prelude {
    //! Common traits and macros for building and querying models.
    //!
    //! ```
    //! # include!("doctest_setup.rs");
    //! # fn main() -> Result<(), Box<dyn std::error::Error>> {
    //! use schema::*;
    //!
    //! let mut conn = connection()?;
    //! let mut builder = users::table::try_builder()?.name("Ada").try_age(20)?;
    //! builder.name_ref("Grace").try_age_ref(30)?;
    //! assert_eq!(builder.try_age_ref(17).err(), Some(ValidationError::AgeTooYoung));
    //! let user = builder.insert(&mut conn)?;
    //! assert_eq!(user.name(), "Grace");
    //! assert_eq!(*user.age(), 30);
    //! let post = posts::table::builder().user_id(user.id).title("Builders").insert(&mut conn)?;
    //! assert_eq!(post.user(&mut conn)?.id, user.id);
    //! let link = links::table::builder().owner(user.id).insert(&mut conn)?;
    //! assert_eq!(link.owner_fk(&mut conn)?.name(), "Grace");
    //! # Ok(())
    //! # }
    //! ```
    //!
    //! ```
    //! # include!("doctest_setup.rs");
    //! # fn main() -> Result<(), Box<dyn std::error::Error>> {
    //! use schema::*;
    //!
    //! let mut conn = connection()?;
    //! let mut builder = children::table::builder();
    //! builder.mandatory_ref(sides::table::builder().label("first"));
    //! builder.discretionary_ref(sides::table::builder().label("next"));
    //! let builder = builder
    //!     .mandatory(sides::table::builder())
    //!     .discretionary(sides::table::builder())
    //!     .child_label("short");
    //! let child = builder.insert(&mut conn)?;
    //! assert_eq!(child.mandatory(&mut conn)?.label, "short");
    //! assert_eq!(child.discretionary(&mut conn)?.parent_id, child.id);
    //!
    //! let side = Side { id: 7, parent_id: 3, label: "linked".into() };
    //! let mut linked = children::table::builder();
    //! linked.discretionary_model_ref(&side);
    //! assert_eq!(linked.may_get_column::<children::discretionary_id>(), Some(7));
    //! let side = Side { id: 8, parent_id: 4, label: "other".into() };
    //! let linked = linked.discretionary_model(&side);
    //! assert_eq!(linked.may_get_column::<children::id>(), Some(4));
    //! assert_eq!(linked.may_get_column::<children::child_label>().as_deref(), Some("other"));
    //! # Ok(())
    //! # }
    //! ```
    //!
    //! ```
    //! # include!("doctest_setup.rs");
    //! # fn main() -> Result<(), Box<dyn std::error::Error>> {
    //! use schema::*;
    //!
    //! let mut conn = connection()?;
    //! let mut builder = checked_children::table::builder();
    //! builder.try_mandatory_ref(checked_sides::table::builder())?;
    //! builder.try_discretionary_ref(checked_sides::table::builder())?;
    //! let mut builder = builder
    //!     .try_mandatory(checked_sides::table::builder())?
    //!     .try_discretionary(checked_sides::table::builder())?
    //!     .try_child_label("short")?;
    //! assert_eq!(builder.try_child_label_ref("longer").err(), Some(ValidationError::LabelTooLong));
    //! let child = builder.insert(&mut conn)?;
    //! assert_eq!(child.child_label, "short");
    //! assert_eq!(child.mandatory(&mut conn)?.label, "short");
    //!
    //! let side = CheckedSide { id: 7, parent_id: 3, label: "short".into() };
    //! let mut linked = checked_children::table::builder()
    //!     .try_mandatory(checked_sides::table::builder())?
    //!     .try_child_label("short")?;
    //! linked.try_discretionary_model_ref(&side)?;
    //! let saved = linked.clone();
    //! let rejected = CheckedSide { id: 8, parent_id: 4, label: "deny".into() };
    //! assert_eq!(
    //!     linked.try_discretionary_model_ref(&rejected).err(),
    //!     Some(ValidationError::ReservedLabel)
    //! );
    //! assert_eq!(linked, saved);
    //! assert_eq!(linked.may_get_column::<checked_children::discretionary_id>(), Some(7));
    //! let side = CheckedSide { id: 9, parent_id: 5, label: "next".into() };
    //! let linked = linked.try_discretionary_model(&side)?;
    //! assert_eq!(linked.may_get_column::<checked_children::id>(), Some(5));
    //! assert_eq!(linked.may_get_column::<checked_children::child_label>().as_deref(), Some("next"));
    //! # Ok(())
    //! # }
    //! ```

    // Re-export diesel prelude for convenience
    pub use diesel::prelude::*;
    // Table model trait - not exported to avoid collision with TableModel macro
    // pub use crate::table_model::TableModel;

    // Re-export commonly used macros from diesel_builders_derive
    // Note: GetColumn is now automatically implemented by TableModel derive
    pub use diesel_builders_derive::{TableModel, index, unique_index};

    // Table relationship traits
    pub use crate::ancestors::{
        Descendant, DescendantOf, ModelDescendantExt, ModelFind, ModelUpsert,
    };
    // Core table building traits
    pub use crate::buildable_table::BuildableTable;
    // Column accessor extension traits (always use Ext variants)
    pub use crate::get_column::{
        GetColumnExt, MayGetColumnExt, TryGetDynamicColumn, TryGetDynamicColumns,
    };
    // Note: Root is NOT exported here to avoid collision with Root macro from
    // diesel_builders_derive
    pub use crate::horizontal_same_as::HorizontalKey;
    // Query loading traits
    pub use crate::load_query_builder::{LoadFirst, LoadMany, LoadSorted};
    pub use crate::{
        builder_bundle::BundlableTable,
        foreign_key::IterForeignKeyExt,
        get_foreign::GetForeignExt,
        get_model::{GetModelExt, GetNestedModelExt},
        helper_type::NestedModel,
        load_nested_query_builder::{LoadNestedFirst, LoadNestedMany, LoadNestedSorted},
        nested_insert::Insert,
        set_builder::{
            SetDiscretionaryBuilderExt, SetDiscretionaryModelExt, SetMandatoryBuilderExt,
            TrySetDiscretionaryBuilderExt, TrySetDiscretionaryModelExt, TrySetMandatoryBuilderExt,
        },
        set_column::{SetColumnExt, TrySetColumnExt, TrySetDynamicColumn, ValidateColumn},
        table_addition::TableExt,
        table_builder::TableBuilder,
        validation::ValidateRecord,
    };
}
