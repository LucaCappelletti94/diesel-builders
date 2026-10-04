//! Submodule defining the `TableBuilderBundle` struct, which bundles
//! a table record new values and its mandatory and discretionary associated
//! builders.

use std::convert::Infallible;

use diesel::{Column, associations::HasTable};

mod completed_table_builder_bundle;
mod serde;
pub use completed_table_builder_bundle::{CompletedTableBuilderBundle, RecursiveBundleInsert};
use tuplities::prelude::*;

use crate::{
    BuildableTable, ColumnTyped, Columns, DiscretionarySameAsIndex, HasNestedTables,
    HorizontalNestedKeys, MandatorySameAsIndex, MayGetColumn, NestedBuildableTables,
    NestedTableModels, NestedTables, SetColumn, SetDiscretionaryBuilder,
    SetDiscretionarySameAsNestedColumns, SetMandatoryBuilder, SetMandatorySameAsNestedColumns,
    TableBuilder, TableExt, TrySetColumn, TrySetDiscretionaryBuilder,
    TrySetDiscretionarySameAsColumn, TrySetMandatoryBuilder, TrySetMandatorySameAsColumn,
    TupleGetNestedColumns, TupleMayGetNestedColumns, TypedColumn, TypedNestedTuple, ValidateColumn,
    ValidateStoredValues, columns::NestedColumns,
    horizontal_same_as_group::HorizontalSameAsGroupExt, tables::NonCompositePrimaryKeyNestedTables,
};

/// Trait representing a Diesel table with associated mandatory and
/// discretionary triangular same-as columns.
pub trait BundlableTable: Sized {
    /// The columns defining mandatory triangular same-as.
    type MandatoryTriangularColumns: Columns<Nested: HorizontalNestedKeys<Self>>;
    /// The columns defining discretionary triangular same-as.
    type DiscretionaryTriangularColumns: Columns<Nested: HorizontalNestedKeys<Self>>;
}

/// Extension trait for [`BundlableTable`].
pub trait BundlableTableExt:
    BundlableTable + TableExt<NewValues: NestedTupleOption<Transposed = Self::CompletedNewValues>>
{
    /// The completed new values ready to be inserted for the table.
    type CompletedNewValues: FlattenNestedTuple
        + IntoNestedTupleOption<IntoOptions = Self::NewValues>;
    /// Nested mandatory triangular same-as columns.
    type NestedMandatoryTriangularColumns: NestedColumns<
        NestedTupleColumnType = Self::NestedMandatoryPrimaryKeyTypes,
    >;
    /// Nested mandatory tables.
    type NestedMandatoryTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns = Self::NestedMandatoryPrimaryKeys,
            NestedModels = Self::NestedMandatoryModels,
        > + NestedBuildableTables;
    /// Nested discretionary triangular same-as columns.
    type NestedDiscretionaryTriangularColumns: NestedColumns<
        NestedTupleColumnType = Self::NestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested discretionary tables.
    type NestedDiscretionaryTables: NonCompositePrimaryKeyNestedTables<
            NestedPrimaryKeyColumns = Self::NestedDiscretionaryPrimaryKeys,
            NestedModels = Self::NestedDiscretionaryModels,
            OptionalNestedModels = Self::OptionalNestedDiscretionaryModels,
        > + NestedBuildableTables;
    /// Nested mandatory foreign primary keys.
    type NestedMandatoryPrimaryKeys: NestedColumns<
        NestedTupleColumnType = Self::NestedMandatoryPrimaryKeyTypes,
    >;
    /// Nested mandatory foreign primary keys types.
    type NestedMandatoryPrimaryKeyTypes;
    /// Nested discretionary foreign primary keys.
    type NestedDiscretionaryPrimaryKeys: NestedColumns<
        NestedTupleColumnType = Self::NestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested discretionary foreign primary key types.
    type NestedDiscretionaryPrimaryKeyTypes: IntoNestedTupleOption<
        IntoOptions = Self::OptionalNestedDiscretionaryPrimaryKeyTypes,
    >;
    /// Nested optional discretionary foreign primary key types.
    type OptionalNestedDiscretionaryPrimaryKeyTypes;
    /// Builders for the mandatory associated tables.
    type MandatoryNestedBuilders: IntoNestedTupleOption<IntoOptions = Self::OptionalMandatoryNestedBuilders>
        + HasNestedTables<NestedTables = Self::NestedMandatoryTables>;
    /// Optional builders for the mandatory associated tables.
    type OptionalMandatoryNestedBuilders: NestedTupleOptionWith<
            &'static str,
            SameDepth = <Self::NestedMandatoryTriangularColumns as NestedColumns>::NestedNames,
            Transposed = Self::MandatoryNestedBuilders,
        > + HasNestedTables<NestedTables = Self::NestedMandatoryTables>;
    /// Builders for the discretionary associated tables.
    type DiscretionaryNestedBuilders: IntoNestedTupleOption<IntoOptions = Self::OptionalDiscretionaryNestedBuilders>
        + HasNestedTables<NestedTables = Self::NestedDiscretionaryTables>;
    /// Optional builders for the discretionary associated tables.
    type OptionalDiscretionaryNestedBuilders: NestedTupleOption<Transposed = Self::DiscretionaryNestedBuilders>
        + HasNestedTables<NestedTables = Self::NestedDiscretionaryTables>;
    /// The nested mandatory models.
    type NestedMandatoryModels: NestedTableModels<NestedTables = Self::NestedMandatoryTables>
        + TupleGetNestedColumns<Self::NestedMandatoryPrimaryKeys>;
    /// The nested discretionary models.
    type NestedDiscretionaryModels: NestedTableModels<
            IntoOptions = Self::OptionalNestedDiscretionaryModels,
            NestedTables = Self::NestedDiscretionaryTables,
        > + TupleGetNestedColumns<Self::NestedDiscretionaryPrimaryKeys>;
    /// The nested optional discretionary models.
    type OptionalNestedDiscretionaryModels: TupleMayGetNestedColumns<
        Self::NestedDiscretionaryPrimaryKeys,
    >;
}

impl<T> BundlableTableExt for T
where
    T: BundlableTable + TableExt,
{
    type CompletedNewValues = <T::NewRecord as TypedNestedTuple>::NestedTupleColumnType;
    type NestedMandatoryTriangularColumns = <T::MandatoryTriangularColumns as NestTuple>::Nested;
    type NestedMandatoryTables =
        <Self::NestedMandatoryTriangularColumns as HorizontalNestedKeys<T>>::NestedReferencedTables;
    type NestedDiscretionaryTriangularColumns =
        <T::DiscretionaryTriangularColumns as NestTuple>::Nested;
    type NestedDiscretionaryTables =
        <Self::NestedDiscretionaryTriangularColumns as HorizontalNestedKeys<
            T,
        >>::NestedReferencedTables;
    type NestedMandatoryPrimaryKeys =
        <Self::NestedMandatoryTables as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns;
    type NestedMandatoryPrimaryKeyTypes =
        <Self::NestedMandatoryPrimaryKeys as TypedNestedTuple>::NestedTupleColumnType;
    type NestedDiscretionaryPrimaryKeys =
        <Self::NestedDiscretionaryTables as NonCompositePrimaryKeyNestedTables>::NestedPrimaryKeyColumns;
    type NestedDiscretionaryPrimaryKeyTypes =
        <Self::NestedDiscretionaryPrimaryKeys as TypedNestedTuple>::NestedTupleColumnType;
    type OptionalNestedDiscretionaryPrimaryKeyTypes =
        <Self::NestedDiscretionaryPrimaryKeyTypes as IntoNestedTupleOption>::IntoOptions;
    type MandatoryNestedBuilders =
        <Self::NestedMandatoryTables as NestedBuildableTables>::NestedBuilders;
    type OptionalMandatoryNestedBuilders =
        <Self::MandatoryNestedBuilders as IntoNestedTupleOption>::IntoOptions;
    type DiscretionaryNestedBuilders =
        <Self::NestedDiscretionaryTables as NestedBuildableTables>::NestedBuilders;
    type OptionalDiscretionaryNestedBuilders =
        <Self::DiscretionaryNestedBuilders as IntoNestedTupleOption>::IntoOptions;
    type NestedMandatoryModels = <Self::NestedMandatoryTables as NestedTables>::NestedModels;
    type NestedDiscretionaryModels =
        <Self::NestedDiscretionaryTables as NestedTables>::NestedModels;
    type OptionalNestedDiscretionaryModels =
        <Self::NestedDiscretionaryModels as IntoNestedTupleOption>::IntoOptions;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// A bundle of a table's insertable model and its associated builders.
pub struct TableBuilderBundle<T: BundlableTableExt> {
    /// The insertable model for the table.
    insertable_model: T::NewValues,
    /// The mandatory associated builders relative to triangular same-as.
    nested_mandatory_associated_builders: T::OptionalMandatoryNestedBuilders,
    /// The discretionary associated builders relative to triangular same-as.
    nested_discretionary_associated_builders: T::OptionalDiscretionaryNestedBuilders,
}

/// The validation error of a table's stored new values.
type StoredValuesError<T> =
    <<T as TableExt>::NewValues as ValidateStoredValues<<T as TableExt>::NewRecord>>::Error;

impl<T: BundlableTableExt> TableBuilderBundle<T> {
    /// Creates a bundle with checked empty values and no associated builders.
    ///
    /// The checked empty state carries no declared defaults, so it is
    /// available even when a declared default is invalid.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, TableBuilderBundle};
    /// use schema::*;
    ///
    /// let bundle = TableBuilderBundle::<users::table>::empty();
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column_ref(&bundle), None);
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column_ref(&bundle), None);
    /// let nickname = MayGetColumn::<users::nickname>::may_get_column_ref(&bundle);
    /// assert_eq!(nickname, Some(&None::<String>));
    ///
    /// let bundle = TableBuilderBundle::<invalid_defaults::table>::empty();
    /// assert_eq!(MayGetColumn::<invalid_defaults::value>::may_get_column_ref(&bundle), None,);
    /// # }
    /// ```
    #[must_use]
    pub fn empty() -> Self
    where
        T::OptionalMandatoryNestedBuilders: Default,
        T::OptionalDiscretionaryNestedBuilders: Default,
    {
        Self::from_checked_model(T::empty_new_values())
    }

    /// Creates a bundle around a checked insertable model with no associated
    /// builders.
    #[must_use]
    pub(crate) fn from_checked_model(insertable_model: T::NewValues) -> Self
    where
        T::OptionalMandatoryNestedBuilders: Default,
        T::OptionalDiscretionaryNestedBuilders: Default,
    {
        Self {
            insertable_model,
            nested_mandatory_associated_builders: Default::default(),
            nested_discretionary_associated_builders: Default::default(),
        }
    }

    /// Validates raw values, returning the consumed input on rejection.
    ///
    /// # Errors
    ///
    /// Returns the rejected values and their intrinsic validation error.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, SetColumn, TableBuilderBundle};
    /// use schema::*;
    ///
    /// let mut raw = invalid_defaults::table::empty_new_values();
    /// SetColumn::<invalid_defaults::value>::set_column(&mut raw, 5);
    /// let accepted = TableBuilderBundle::<invalid_defaults::table>::try_from_values(raw);
    /// let Ok(bundle) = accepted else {
    ///     panic!("valid values must be accepted");
    /// };
    /// assert_eq!(MayGetColumn::<invalid_defaults::value>::may_get_column_ref(&bundle), Some(&5),);
    ///
    /// let mut raw = invalid_defaults::table::empty_new_values();
    /// SetColumn::<invalid_defaults::value>::set_column(&mut raw, -5);
    /// let rejected = TableBuilderBundle::<invalid_defaults::table>::try_from_values(raw.clone());
    /// let Err((returned, error)) = rejected else {
    ///     panic!("invalid values must be rejected");
    /// };
    /// assert_eq!(returned, raw);
    /// assert_eq!(error, ValidationError::NegativeValue);
    /// # }
    /// ```
    pub fn try_from_values(
        values: T::NewValues,
    ) -> Result<Self, (T::NewValues, StoredValuesError<T>)>
    where
        T::NewValues: ValidateStoredValues<T::NewRecord>,
        T::OptionalMandatoryNestedBuilders: Default,
        T::OptionalDiscretionaryNestedBuilders: Default,
    {
        if let Err(error) = values.validate_stored_values() {
            return Err((values, error));
        }
        Ok(Self::from_checked_model(values))
    }
}

impl<T> Default for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    crate::DefaultBundle<T>: crate::BuildDefaults<CheckedBundles = Self, Error: Into<Infallible>>,
    T::OptionalMandatoryNestedBuilders: Default,
    T::OptionalDiscretionaryNestedBuilders: Default,
{
    fn default() -> Self {
        let mut staged = crate::DefaultBundle::<T>::new();
        crate::BuildDefaults::check_defaults(&mut staged)
            .map_err(Into::<Infallible>::into)
            .unwrap_or_else(|error| match error {});
        crate::BuildDefaults::into_checked(staged)
    }
}

/// A prepared column split between the owned and the same-as group parts.
pub(crate) struct PreparedBundleColumn<Own, Mandatory, Discretionary> {
    /// The prepared column value of the table's own insertable model.
    own: Own,
    /// The prepared mandatory horizontal same-as group.
    mandatory: Mandatory,
    /// The prepared discretionary horizontal same-as group.
    discretionary: Discretionary,
}

impl<C: TypedColumn, Own: crate::mutation::ColumnInput<C>, Mandatory, Discretionary>
    crate::mutation::ColumnInput<C> for PreparedBundleColumn<Own, Mandatory, Discretionary>
{
    fn value(&self) -> &C::ColumnType {
        self.own.value()
    }

    fn into_value(self) -> C::ColumnType {
        self.own.into_value()
    }
}

/// Emits the trait impls shared by the table builder bundle variants.
macro_rules! impl_bundle_shared {
    ($bundle:ident) => {
        impl<T> HasTable for $bundle<T>
        where
            T: BundlableTableExt,
        {
            type Table = T;

            #[inline]
            fn table() -> Self::Table {
                T::default()
            }
        }

        impl<T, C> ValidateColumn<C> for $bundle<T>
        where
            T: BundlableTableExt,
            C: TypedColumn<Table = T>,
            T::NewValues: ValidateColumn<C>,
        {
            type Error = <T::NewValues as ValidateColumn<C>>::Error;

            #[inline]
            fn validate_column(value: &C::ValueType) -> Result<(), Self::Error> {
                <T::NewValues as ValidateColumn<C>>::validate_column(value)
            }
        }

        impl<T, C> crate::mutation::PrepareColumn<C> for $bundle<T>
        where
            T: BundlableTableExt,
            C: HorizontalSameAsGroupExt<Table = T>,
            T::NewValues: crate::mutation::PrepareColumn<C>,
            Self: crate::mutation_same_as::PrepareMandatoryGroup<
                    Self::Error, C::ValueType, C::NestedMandatoryHorizontalKeys,
                    C::NestedMandatoryForeignColumns,
                > + crate::mutation_same_as::PrepareDiscretionaryGroup<
                    Self::Error, C::ValueType, C::NestedDiscretionaryHorizontalKeys,
                    C::NestedDiscretionaryForeignColumns,
                >,
        {
            type Prepared = crate::builder_bundle::PreparedBundleColumn<
                <T::NewValues as crate::mutation::PrepareColumn<C>>::Prepared,
                <Self as crate::mutation_same_as::PrepareMandatoryGroup<
                    Self::Error, C::ValueType, C::NestedMandatoryHorizontalKeys,
                    C::NestedMandatoryForeignColumns,
                >>::Prepared,
                <Self as crate::mutation_same_as::PrepareDiscretionaryGroup<
                    Self::Error, C::ValueType, C::NestedDiscretionaryHorizontalKeys,
                    C::NestedDiscretionaryForeignColumns,
                >>::Prepared,
            >;

            fn prepare_column(&self, value: C::ColumnType, context: &crate::mutation::MutationContext)
                -> Result<Self::Prepared, (C::ColumnType, Self::Error)>
            {
                use crate::mutation::ColumnInput;
                use crate::mutation_same_as::{PrepareMandatoryGroup, PrepareDiscretionaryGroup};
                let own = <T::NewValues as crate::mutation::PrepareColumn<C>>::prepare_column(
                    &self.insertable_model, value, context,
                )?;
                let mandatory = match self.prepare_mandatory_group(own.value(), context) {
                    Ok(prepared) => prepared,
                    Err(error) => return Err((own.into_value(), error)),
                };
                let discretionary = match self.prepare_discretionary_group(own.value(), context) {
                    Ok(prepared) => prepared,
                    Err(error) => return Err((own.into_value(), error)),
                };
                Ok(crate::builder_bundle::PreparedBundleColumn { own, mandatory, discretionary })
            }

            fn apply_column(&mut self, prepared: Self::Prepared) {
                use crate::mutation_same_as::{PrepareMandatoryGroup, PrepareDiscretionaryGroup};
                self.apply_mandatory_group(prepared.mandatory);
                self.apply_discretionary_group(prepared.discretionary);
                <T::NewValues as crate::mutation::PrepareColumn<C>>::apply_column(
                    &mut self.insertable_model, prepared.own,
                );
            }
        }

        impl<T, C> TrySetColumn<C> for $bundle<T>
        where
            T: BundlableTableExt,
            C: TypedColumn<Table = T>,
            Self: crate::mutation::PrepareColumn<C>,
        {
            #[inline]
            fn try_set_column(&mut self, value: impl Into<C::ColumnType>)
                -> Result<&mut Self, Self::Error>
            {
                let prepared = <Self as crate::mutation::PrepareColumn<C>>::prepare_column(
                    self, value.into(), &crate::mutation::MutationContext::default(),
                ).map_err(|(_, error)| error)?;
                <Self as crate::mutation::PrepareColumn<C>>::apply_column(self, prepared);
                Ok(self)
            }
        }

        impl<Key, C> crate::mutation_same_as::PrepareDiscretionaryColumn<Key, C>
            for $bundle<<Key as Column>::Table>
        where
            Key: DiscretionarySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>,
            C: TypedColumn<Table = Key::ReferencedTable>,
            <Key::Table as BundlableTableExt>::OptionalDiscretionaryNestedBuilders:
                NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
            TableBuilder<C::Table>: crate::mutation::PrepareColumn<C>,
        {
            type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;
            type Prepared = Option<<TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::Prepared>;

            fn has_discretionary_target(&self, context: &crate::mutation::MutationContext) -> bool {
                !context.excludes_discretionary(std::ptr::from_ref(self).cast::<()>(), <Key::Idx as typenum::Unsigned>::USIZE)
                    && self.nested_discretionary_associated_builders.nested_index().is_some()
            }

            fn prepare_discretionary_column(&self, value: C::ColumnType, context: &crate::mutation::MutationContext)
                -> Result<Self::Prepared, (C::ColumnType, Self::Error)>
            {
                self.nested_discretionary_associated_builders.nested_index().as_ref()
                    .map(|builder| <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::prepare_column(builder, value, context))
                    .transpose()
            }

            fn apply_discretionary_column(&mut self, prepared: Self::Prepared) {
                if let Some(prepared) = prepared {
                    let builder = self.nested_discretionary_associated_builders.nested_index_mut()
                        .as_mut().expect("prepared discretionary target remains attached");
                    <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::apply_column(builder, prepared);
                }
            }
        }

        impl<
            Key: DiscretionarySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>,
            C,
        > TrySetDiscretionarySameAsColumn<Key, C> for $bundle<<Key as Column>::Table>
        where
            C: TypedColumn<Table = Key::ReferencedTable>,
            <Key::Table as BundlableTableExt>::OptionalDiscretionaryNestedBuilders:
                NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
            TableBuilder<C::Table>: TrySetColumn<C>,
        {
            type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;

            #[inline]
            fn try_set_discretionary_same_as_column(
                &mut self,
                value: impl Into<C::ColumnType>,
            ) -> Result<&mut Self, Self::Error> {
                if let Some(builder) =
                    self.nested_discretionary_associated_builders.nested_index_mut().as_mut()
                {
                    builder.try_set_column(value)?;
                }
                Ok(self)
            }
        }
    };
}
pub(crate) use impl_bundle_shared;

impl_bundle_shared!(TableBuilderBundle);

impl<T, C> MayGetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: ColumnTyped,
    T::NewValues: MayGetColumn<C>,
{
    #[inline]
    fn may_get_column_ref(&self) -> Option<&C::ColumnType> {
        self.insertable_model.may_get_column_ref()
    }
}

impl<T, C> SetColumn<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: HorizontalSameAsGroupExt<Table = T>,
    Self: SetDiscretionarySameAsNestedColumns<
            C::ValueType,
            C::NestedDiscretionaryHorizontalKeys,
            C::NestedDiscretionaryForeignColumns,
        > + SetMandatorySameAsNestedColumns<
            C::ValueType,
            C::NestedMandatoryHorizontalKeys,
            C::NestedMandatoryForeignColumns,
        >,
    T::NewValues: SetColumn<C> + ValidateColumn<C, Error = std::convert::Infallible>,
{
    #[inline]
    fn set_column(&mut self, value: impl Into<C::ColumnType>) -> &mut Self {
        let value = value.into();
        self.set_discretionary_same_as_nested_columns(&value);
        self.set_mandatory_same_as_nested_columns(&value);
        self.insertable_model.set_column(value);
        self
    }
}

impl<Key: MandatorySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>, C>
    TrySetMandatorySameAsColumn<Key, C> for TableBuilderBundle<<Key as Column>::Table>
where
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::OptionalMandatoryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
    TableBuilder<C::Table>: TrySetColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;

    #[inline]
    fn try_set_mandatory_same_as_column(
        &mut self,
        value: impl Into<C::ColumnType>,
    ) -> Result<&mut Self, Self::Error> {
        if let Some(builder) = self.nested_mandatory_associated_builders.nested_index_mut() {
            builder.try_set_column(value)?;
        }
        Ok(self)
    }
}

impl<Key, C> crate::mutation_same_as::PrepareMandatoryColumn<Key, C>
    for TableBuilderBundle<<Key as Column>::Table>
where
    Key: MandatorySameAsIndex<Table: BundlableTableExt, ReferencedTable: BuildableTable>,
    C: TypedColumn<Table = Key::ReferencedTable>,
    <Key::Table as BundlableTableExt>::OptionalMandatoryNestedBuilders:
        NestedTupleIndexMut<Key::Idx, Element = Option<TableBuilder<C::Table>>>,
    TableBuilder<C::Table>: crate::mutation::PrepareColumn<C>,
{
    type Error = <TableBuilder<C::Table> as ValidateColumn<C>>::Error;
    type Prepared = Option<<TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::Prepared>;

    fn has_mandatory_target(&self, context: &crate::mutation::MutationContext) -> bool {
        !context.excludes_mandatory(
            std::ptr::from_ref(self).cast::<()>(),
            <Key::Idx as typenum::Unsigned>::USIZE,
        ) && self.nested_mandatory_associated_builders.nested_index().is_some()
    }

    fn prepare_mandatory_column(
        &self,
        value: C::ColumnType,
        context: &crate::mutation::MutationContext,
    ) -> Result<Self::Prepared, (C::ColumnType, Self::Error)> {
        self.nested_mandatory_associated_builders
            .nested_index()
            .as_ref()
            .map(|builder| {
                <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::prepare_column(
                    builder, value, context,
                )
            })
            .transpose()
    }

    fn apply_mandatory_column(&mut self, prepared: Self::Prepared) {
        if let Some(prepared) = prepared {
            #[expect(
                clippy::expect_used,
                reason = "prepare and apply never change mandatory target membership"
            )]
            let builder = self
                .nested_mandatory_associated_builders
                .nested_index_mut()
                .as_mut()
                .expect("prepared mandatory target remains attached");
            <TableBuilder<C::Table> as crate::mutation::PrepareColumn<C>>::apply_column(
                builder, prepared,
            );
        }
    }
}

impl<C, T> SetMandatoryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: MandatorySameAsIndex,
    C::ReferencedTable: BuildableTable,
    T::OptionalMandatoryNestedBuilders: NestedTupleIndexMut<
            <C as MandatorySameAsIndex>::Idx,
            Element = Option<TableBuilder<C::ReferencedTable>>,
        >,
{
    #[inline]
    fn set_mandatory_builder(&mut self, builder: TableBuilder<C::ReferencedTable>) -> &mut Self {
        *self.nested_mandatory_associated_builders.nested_index_mut() = Some(builder);
        self
    }
}

impl<Key> TrySetMandatoryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTableExt,
    Key: MandatorySameAsIndex,
    Key::ReferencedTable: BuildableTable,
    <Key::Table as BundlableTableExt>::OptionalMandatoryNestedBuilders: NestedTupleIndexMut<
            <Key as MandatorySameAsIndex>::Idx,
            Element = Option<TableBuilder<Key::ReferencedTable>>,
        >,
{
    #[inline]
    fn try_set_mandatory_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error> {
        *self.nested_mandatory_associated_builders.nested_index_mut() = Some(builder);
        Ok(self)
    }
}

impl<C, T> SetDiscretionaryBuilder<C> for TableBuilderBundle<T>
where
    T: BundlableTableExt,
    C: DiscretionarySameAsIndex,
    C::ReferencedTable: BuildableTable,
    T::OptionalDiscretionaryNestedBuilders: NestedTupleIndexMut<
            <C as DiscretionarySameAsIndex>::Idx,
            Element = Option<TableBuilder<C::ReferencedTable>>,
        >,
{
    #[inline]
    fn set_discretionary_builder(
        &mut self,
        builder: TableBuilder<C::ReferencedTable>,
    ) -> &mut Self {
        *self.nested_discretionary_associated_builders.nested_index_mut() = Some(builder);
        self
    }
}

impl<Key> TrySetDiscretionaryBuilder<Key> for TableBuilderBundle<Key::Table>
where
    Key::Table: BundlableTableExt,
    Key: DiscretionarySameAsIndex,
    Key::ReferencedTable: BuildableTable,
    <Key::Table as BundlableTableExt>::OptionalDiscretionaryNestedBuilders: NestedTupleIndexMut<
            <Key as DiscretionarySameAsIndex>::Idx,
            Element = Option<TableBuilder<Key::ReferencedTable>>,
        >,
{
    #[inline]
    fn try_set_discretionary_builder(
        &mut self,
        builder: TableBuilder<Key::ReferencedTable>,
    ) -> Result<&mut Self, <Self::Table as TableExt>::Error> {
        *self.nested_discretionary_associated_builders.nested_index_mut() = Some(builder);
        Ok(self)
    }
}
