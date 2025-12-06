//! Submodule implementing core-traits for table bundles.

use tuplities::prelude::*;

use crate::{BundlableTable, HorizontalSameAsKeys, TableBuilderBundle};

impl<T: BundlableTable> Clone for TableBuilderBundle<T>
where
    T::InsertableModel: Clone,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleClone,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleClone,
{
    fn clone(&self) -> Self {
        Self {
            insertable_model: self.insertable_model.clone(),
            mandatory_associated_builders: self.mandatory_associated_builders.tuple_clone(),
            discretionary_associated_builders: self.discretionary_associated_builders.tuple_clone(),
        }
    }
}

impl<T: BundlableTable> Copy for TableBuilderBundle<T>
where
    T::InsertableModel: Copy,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: Copy + TupleCopy,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: Copy + TupleCopy,
{
}

impl<T: BundlableTable> PartialEq for TableBuilderBundle<T>
where
    T::InsertableModel: PartialEq,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialEq,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.insertable_model == other.insertable_model
            && self.mandatory_associated_builders.tuple_eq(&other.mandatory_associated_builders)
            && self.discretionary_associated_builders.tuple_eq(&other.discretionary_associated_builders)
    }
}

impl<T: BundlableTable> Eq for TableBuilderBundle<T>
where
    T::InsertableModel: Eq,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleEq,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleEq,
{
}

impl<T: BundlableTable> core::hash::Hash for TableBuilderBundle<T>
where
    T::InsertableModel: core::hash::Hash,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleHash,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleHash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.insertable_model.hash(state);
        self.mandatory_associated_builders.tuple_hash(state);
        self.discretionary_associated_builders.tuple_hash(state);
    }
}

impl<T: BundlableTable> PartialOrd for TableBuilderBundle<T>
where
    T::InsertableModel: PartialOrd,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialOrd,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TuplePartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        match self.insertable_model.partial_cmp(&other.insertable_model) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.mandatory_associated_builders.tuple_partial_cmp(&other.mandatory_associated_builders) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.discretionary_associated_builders.tuple_partial_cmp(&other.discretionary_associated_builders)
    }
}
impl<T: BundlableTable> Ord for TableBuilderBundle<T>
where
    T::InsertableModel: Ord,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleOrd,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleOrd,
{
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        match self.insertable_model.cmp(&other.insertable_model) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        match self.mandatory_associated_builders.tuple_cmp(&other.mandatory_associated_builders) {
            core::cmp::Ordering::Equal => {}
            ord => return ord,
        }
        self.discretionary_associated_builders.tuple_cmp(&other.discretionary_associated_builders)
    }
}

impl<T: BundlableTable> core::fmt::Debug for TableBuilderBundle<T>
where
    T::InsertableModel: core::fmt::Debug,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDebug,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDebug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TableBuilderBundle")
            .field("insertable_model", &self.insertable_model)
            .field(
                "mandatory_associated_builders",
                &self.mandatory_associated_builders.tuple_debug(),
            )
            .field(
                "discretionary_associated_builders",
                &self.discretionary_associated_builders.tuple_debug(),
            )
            .finish()
    }
}

impl<T> Default for TableBuilderBundle<T>
where
    T: BundlableTable,
    T::InsertableModel: Default,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDefault,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: TupleDefault,
{
    fn default() -> Self {
        Self {
            insertable_model: Default::default(),
            mandatory_associated_builders: TupleDefault::tuple_default(),
            discretionary_associated_builders: TupleDefault::tuple_default(),
        }
    }
}
