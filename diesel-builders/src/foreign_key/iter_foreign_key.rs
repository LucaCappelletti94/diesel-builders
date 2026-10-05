//! Submodule defining a trait to iterate the foreign keys in a table
//! which reference the same foreign index in another table.

use tuplities::prelude::{IntoNestedTupleOption, NestedTupleOption, NestedTupleRef};

use crate::{
    TryGetDynamicColumns, TypedNestedTuple,
    builder_error::DynamicColumnError,
    columns::{
        HasNestedDynColumns, NestedDynColumns, NonEmptyNestedProjection, NonEmptyProjection,
    },
    get_column::dynamic_multi::sealed::VariadicTryGetDynamicColumns,
};

mod blankets;

/// Alias for the reference type of a nested tuple value.
type Ref<'a, T> = <<T as TypedNestedTuple>::NestedTupleValueType as NestedTupleRef>::Ref<'a>;
/// Alias for the optional reference type of a nested tuple value.
type OptRef<'a, T> = <Ref<'a, T> as IntoNestedTupleOption>::IntoOptions;

/// An iterator over foreign keys in a table which reference the same foreign
/// dynamic index. The index is represented as a nested tuple of dynamic
/// columns.
pub trait IterDynForeignKeys<DynIdx: NestedDynColumns>: TryGetDynamicColumns {
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{DynColumn, IterDynForeignKeys};
    /// use schema::{Child, children, sides};
    ///
    /// let index: (DynColumn<i32>, (DynColumn<i32>,)) = (sides::id.into(), (sides::parent_id.into(),));
    /// let columns: Vec<_> = <Child as IterDynForeignKeys<(DynColumn<i32>, (DynColumn<i32>,))>>::iter_foreign_key_dyn_columns(index).collect();
    /// assert_eq!(columns, vec![
    ///     (children::mandatory_id.into(), (children::id.into(),)),
    ///     (children::discretionary_id.into(), (children::id.into(),)),
    /// ]);
    /// let unrelated: Vec<_> = <Child as IterDynForeignKeys<_>>::iter_foreign_key_dyn_columns((sides::id.into(),)).collect();
    /// assert!(unrelated.is_empty());
    /// # }
    /// ```
    fn iter_foreign_key_dyn_columns(index: DynIdx) -> impl Iterator<Item = DynIdx>;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index, including keys with `None` values.
    ///
    /// # Errors
    ///
    /// As described in the [`IterDynForeignKeys::iter_foreign_key_dyn_columns`]
    /// method, this method is dynamic in nature, and may fail if, due to
    /// antagonistic parameterization of the provided index, the foreign keys
    /// cannot be retrieved.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{DynColumn, IterDynForeignKeys};
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let child: Child = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short")
    ///     .insert(&mut conn)?;
    /// let nested = (child,);
    /// let index: (DynColumn<i32>, (DynColumn<i32>,)) = (sides::id.into(), (sides::parent_id.into(),));
    /// let matches: Vec<_> = nested.iter_dyn_match_simple(index).collect::<Result<Vec<_>, _>>()?;
    /// assert_eq!(
    ///     matches,
    ///     vec![
    ///         (Some(&nested.0.mandatory_id), (Some(&nested.0.id),)),
    ///         (Some(&nested.0.discretionary_id), (Some(&nested.0.id),)),
    ///     ]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn iter_dyn_match_simple<'a>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<OptRef<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: 'a + VariadicTryGetDynamicColumns<'a, Self>,
    {
        Self::iter_foreign_key_dyn_columns(index).map(|keys| self.try_get_dynamic_columns_ref(keys))
    }

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index, skipping keys with `None` values.
    ///
    /// # Errors
    ///
    /// As described in the [`IterDynForeignKeys::iter_foreign_key_dyn_columns`]
    /// method, this method is dynamic in nature, and may fail if, due to
    /// antagonistic parameterization of the provided index, the foreign keys
    /// cannot be retrieved.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::{DynColumn, IterDynForeignKeys};
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let child: Child = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short")
    ///     .insert(&mut conn)?;
    /// let nested = (child,);
    /// let index: (DynColumn<i32>, (DynColumn<i32>,)) = (sides::id.into(), (sides::parent_id.into(),));
    /// let matches: Vec<_> = nested.iter_dyn_match_full(index).collect::<Result<Vec<_>, _>>()?;
    /// assert_eq!(
    ///     matches,
    ///     vec![
    ///         (&nested.0.mandatory_id, (&nested.0.id,)),
    ///         (&nested.0.discretionary_id, (&nested.0.id,)),
    ///     ]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn iter_dyn_match_full<'a>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<Ref<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: 'a + VariadicTryGetDynamicColumns<'a, Self>,
    {
        self.iter_dyn_match_simple(index)
            .filter_map(|res| res.map(NestedTupleOption::transpose).transpose())
    }
}

/// An iterator over foreign keys in a table which reference the same foreign
/// index in another table. The index is represented as a nested tuple of
/// diesel column marker structs.
pub trait IterForeignKeys<NestedIdx: HasNestedDynColumns + NonEmptyNestedProjection> {
    /// Returns an iterator over the foreign keys in this table.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::IterForeignKeys;
    /// use schema::{Post, posts, users};
    ///
    /// let columns: Vec<_> =
    ///     <Post as IterForeignKeys<(users::id,)>>::iter_foreign_key_columns().collect();
    /// assert_eq!(columns, vec![(posts::user_id.into(),)]);
    /// # }
    /// ```
    fn iter_foreign_key_columns()
    -> impl Iterator<Item = <NestedIdx as HasNestedDynColumns>::NestedDynColumns>;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are included.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterDynForeignKeys`] trait instead.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::IterForeignKeys;
    /// use schema::{Post, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let first = Post::find(&1, &mut conn)?;
    /// let matches: Vec<_> =
    ///     <Post as IterForeignKeys<(users::id,)>>::iter_match_simple(&first).collect();
    /// assert_eq!(matches, vec![(Some(&1),)]);
    /// # Ok(())
    /// # }
    /// ```
    fn iter_match_simple<'a>(&'a self) -> impl Iterator<Item = OptRef<'a, NestedIdx>>
    where
        NestedIdx: 'a;

    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index. Foreign keys with `None` values are skipped.
    ///
    /// This method will not be available in table hierarchies if any table in
    /// the hierarchy does not have at least one foreign key referencing the
    /// given foreign index. If you need to handle such cases, consider using
    /// the [`IterDynForeignKeys`] trait instead.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::IterForeignKeys;
    /// use schema::{Post, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let third = Post::find(&3, &mut conn)?;
    /// let matches: Vec<_> =
    ///     <Post as IterForeignKeys<(users::id,)>>::iter_match_full(&third).collect();
    /// assert_eq!(matches, vec![(&2,)]);
    /// # Ok(())
    /// # }
    /// ```
    fn iter_match_full<'a>(&'a self) -> impl Iterator<Item = Ref<'a, NestedIdx>>
    where
        NestedIdx: 'a,
    {
        self.iter_match_simple().filter_map(NestedTupleOption::transpose)
    }
}

/// An extension of the [`IterForeignKeys`] trait moving the generic parameter
/// from the trait to the method to facilitate usage in certain contexts.
pub trait IterForeignKeyExt {
    #[inline]
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index, including keys with `None` values.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let child: Child = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short")
    ///     .insert(&mut conn)?;
    /// assert_eq!(child.id, 1);
    /// let matches: Vec<_> = child.iter_match_simple::<(sides::id, sides::parent_id)>().collect();
    /// assert_eq!(
    ///     matches,
    ///     vec![
    ///         (Some(&child.mandatory_id), (Some(&child.id),)),
    ///         (Some(&child.discretionary_id), (Some(&child.id),)),
    ///     ]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn iter_match_simple<'a, Idx>(&'a self) -> impl Iterator<Item = OptRef<'a, Idx::Nested>>
    where
        Idx: NonEmptyProjection<Nested: HasNestedDynColumns> + 'a,
        Self: IterForeignKeys<Idx::Nested>,
    {
        IterForeignKeys::iter_match_simple(self)
    }

    #[inline]
    /// Returns an iterator over the DYNAMIC foreign keys in this table which
    /// reference the given foreign index, including keys with `None` values.
    ///
    /// # Errors
    ///
    /// Read the documentation of [`IterDynForeignKeys::iter_dyn_match_simple`]
    /// for details on possible errors.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::DynColumn;
    /// use schema::{Post, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let first = Post::find(&1, &mut conn)?;
    /// let nested = (first,);
    /// let index: (DynColumn<i32>,) = (users::id.into(),);
    /// let matches: Vec<_> = nested.iter_dynamic_match_simple(index).collect::<Result<Vec<_>, _>>()?;
    /// assert_eq!(matches, vec![(Some(&1),)]);
    /// # Ok(())
    /// # }
    /// ```
    fn iter_dynamic_match_simple<'a, DynIdx>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<OptRef<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: NestedDynColumns + VariadicTryGetDynamicColumns<'a, Self> + 'a,
        Self: IterDynForeignKeys<DynIdx>,
    {
        IterDynForeignKeys::iter_dyn_match_simple(self, index)
    }

    #[inline]
    /// Returns an iterator over the foreign keys in this table which reference
    /// the given foreign index, skipping keys with `None` values.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let child: Child = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short")
    ///     .insert(&mut conn)?;
    /// let matches: Vec<_> = child.iter_match_full::<(sides::id, sides::parent_id)>().collect();
    /// assert_eq!(
    ///     matches,
    ///     vec![(&child.mandatory_id, (&child.id,)), (&child.discretionary_id, (&child.id,)),]
    /// );
    /// # Ok(())
    /// # }
    /// ```
    fn iter_match_full<'a, Idx>(&'a self) -> impl Iterator<Item = Ref<'a, Idx::Nested>>
    where
        Idx: NonEmptyProjection<Nested: HasNestedDynColumns> + 'a,
        Self: IterForeignKeys<Idx::Nested>,
    {
        <Self as IterForeignKeys<Idx::Nested>>::iter_match_full(self)
    }

    #[inline]
    /// Returns an iterator over the DYNAMIC foreign keys in this table which
    /// reference the given foreign index, skipping keys with `None` values.
    ///
    /// # Errors
    ///
    /// Read the documentation of [`IterDynForeignKeys::iter_dyn_match_full`]
    /// for details on possible errors.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::DynColumn;
    /// use schema::{Post, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let third = Post::find(&3, &mut conn)?;
    /// let nested = (third,);
    /// let index: (DynColumn<i32>,) = (users::id.into(),);
    /// let matches: Vec<_> = nested.iter_dynamic_match_full(index).collect::<Result<Vec<_>, _>>()?;
    /// assert_eq!(matches, vec![(&2,)]);
    /// # Ok(())
    /// # }
    /// ```
    fn iter_dynamic_match_full<'a, DynIdx>(
        &'a self,
        index: DynIdx,
    ) -> impl Iterator<Item = Result<Ref<'a, DynIdx>, DynamicColumnError>>
    where
        DynIdx: NestedDynColumns + VariadicTryGetDynamicColumns<'a, Self> + 'a,
        Self: IterDynForeignKeys<DynIdx>,
    {
        <Self as IterDynForeignKeys<DynIdx>>::iter_dyn_match_full(self, index)
    }

    #[must_use = "iterators are lazy and do nothing unless consumed"]
    #[inline]
    /// Returns an iterator over the foreign keys in this table.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() {
    /// use schema::{Child, children, parents, sides};
    ///
    /// let columns: Vec<_> =
    ///     Child::iter_foreign_key_columns::<(sides::id, sides::parent_id)>().collect();
    /// assert_eq!(
    ///     columns,
    ///     vec![
    ///         (children::mandatory_id.into(), (children::id.into(),)),
    ///         (children::discretionary_id.into(), (children::id.into(),)),
    ///     ]
    /// );
    /// let inherited: Vec<_> = Child::iter_foreign_key_columns::<(parents::id,)>().collect();
    /// assert_eq!(inherited, vec![(children::id.into(),)]);
    /// # }
    /// ```
    fn iter_foreign_key_columns<Idx>()
    -> impl Iterator<Item = <Idx::Nested as HasNestedDynColumns>::NestedDynColumns>
    where
        Idx: NonEmptyProjection<Nested: HasNestedDynColumns>,
        Self: IterForeignKeys<Idx::Nested>,
    {
        <Self as IterForeignKeys<Idx::Nested>>::iter_foreign_key_columns()
    }

    #[inline]
    /// Returns an iterator over the DYNAMIC foreign keys in this table.
    ///
    /// # Errors
    ///
    /// Read the documentation of
    /// [`IterDynForeignKeys::iter_foreign_key_dyn_columns`] for details on
    /// possible errors.
    ///
    /// ```
    /// # include!("../doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::DynColumn;
    /// use schema::{Post, posts, sides, users};
    ///
    /// let index: (DynColumn<i32>,) = (users::id.into(),);
    /// let columns: Vec<_> = Post::iter_dynamic_foreign_key_columns(index).collect();
    /// assert_eq!(columns, vec![(posts::user_id.into(),)]);
    /// let unrelated: Vec<_> = Post::iter_dynamic_foreign_key_columns((sides::id.into(),)).collect();
    /// assert!(unrelated.is_empty());
    /// # }
    /// ```
    fn iter_dynamic_foreign_key_columns<DynIdx>(index: DynIdx) -> impl Iterator<Item = DynIdx>
    where
        DynIdx: NestedDynColumns,
        Self: IterDynForeignKeys<DynIdx>,
    {
        <Self as IterDynForeignKeys<DynIdx>>::iter_foreign_key_dyn_columns(index)
    }
}

impl<T> IterForeignKeyExt for T {}
