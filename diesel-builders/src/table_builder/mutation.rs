use super::{AncestorBundle, AncestorBundleMut, TableBuilder};
use crate::{
    AncestorOfIndex, BuildableTable, DescendantOf, DiscretionarySameAsIndex, MandatorySameAsIndex,
    TableBuilderBundle, TypedColumn,
    builder_bundle::BundlableTableExt,
    mutation::MutationContext,
    mutation_same_as::{PrepareDiscretionaryColumn, PrepareMandatoryColumn},
};

/// Emits the delegating side-preparation impl for one same-as trait.
macro_rules! impl_side_preparation {
    ($trait:ident, $index:ident, $has:ident, $prepare:ident, $apply:ident) => {
        impl<T, Key, C> $trait<Key, C> for TableBuilder<T>
        where
            T: BuildableTable + DescendantOf<Key::Table>,
            Key: $index<Table: BundlableTableExt, ReferencedTable: BuildableTable>,
            Key::Table: AncestorOfIndex<T>,
            C: TypedColumn<Table = Key::ReferencedTable>,
            Self: AncestorBundleMut<Key::Table>,
            TableBuilderBundle<Key::Table>: $trait<Key, C>,
        {
            type Error = <TableBuilderBundle<Key::Table> as $trait<Key, C>>::Error;
            type Prepared = <TableBuilderBundle<Key::Table> as $trait<Key, C>>::Prepared;

            fn $has(&self, context: &MutationContext) -> bool {
                <TableBuilderBundle<Key::Table> as $trait<Key, C>>::$has(
                    self.ancestor_bundle(),
                    context,
                )
            }

            fn $prepare(
                &self,
                value: C::ColumnType,
                context: &MutationContext,
            ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)> {
                <TableBuilderBundle<Key::Table> as $trait<Key, C>>::$prepare(
                    self.ancestor_bundle(),
                    value,
                    context,
                )
            }

            fn $apply(&mut self, prepared: Self::Prepared) {
                <TableBuilderBundle<Key::Table> as $trait<Key, C>>::$apply(
                    self.ancestor_bundle_mut(),
                    prepared,
                );
            }
        }
    };
}

impl_side_preparation!(
    PrepareMandatoryColumn,
    MandatorySameAsIndex,
    has_mandatory_target,
    prepare_mandatory_column,
    apply_mandatory_column
);
impl_side_preparation!(
    PrepareDiscretionaryColumn,
    DiscretionarySameAsIndex,
    has_discretionary_target,
    prepare_discretionary_column,
    apply_discretionary_column
);
