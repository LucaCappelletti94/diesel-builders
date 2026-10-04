//! Submodule providing the `GetColumn` trait.

use diesel::associations::HasTable;
use tuplities::prelude::{NestedTupleIndex, NestedTuplePopBack};

use crate::{AncestorOfIndex, ColumnTyped, DescendantOf, HasTableExt, TypedColumn};

/// Trait providing a getter for a specific Diesel column.
pub trait GetColumn<Column: ColumnTyped> {
    /// Get the value of the specified column.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetColumn;
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user: User = users::table.find(1).first(&mut conn)?;
    /// let name = <User as GetColumn<users::name>>::get_column_ref(&user);
    /// assert_eq!(name, "Ada");
    /// assert!(std::ptr::eq(name, &user.name));
    /// # Ok(())
    /// # }
    /// ```
    fn get_column_ref(&self) -> &Column::ColumnType;
    /// Get the owned value of the specified column.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetColumn;
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user: User = users::table.find(1).first(&mut conn)?;
    /// let mut name = <User as GetColumn<users::name>>::get_column(&user);
    /// assert_eq!(name, "Ada".to_owned());
    /// name.clear();
    /// assert_eq!(<User as GetColumn<users::name>>::get_column(&user), "Ada".to_owned());
    /// assert_eq!(<User as GetColumn<users::nickname>>::get_column(&user), Some("Ace".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    fn get_column(&self) -> Column::ColumnType {
        self.get_column_ref().clone()
    }
}

impl<T, C> GetColumn<C> for (T,)
where
    C: ColumnTyped,
    T: GetColumn<C>,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        self.0.get_column_ref()
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        self.0.get_column()
    }
}

impl<Head, Tail, C> GetColumn<C> for (Head, Tail)
where
    C: TypedColumn,
    Tail: NestedTuplePopBack<Back: HasTableExt<Table: DescendantOf<C::Table>>>,
    C::Table: AncestorOfIndex<<Tail::Back as HasTable>::Table>,
    (Head, Tail): NestedTupleIndex<
            <C::Table as AncestorOfIndex<<Tail::Back as HasTable>::Table>>::Idx,
            Element: GetColumn<C>,
        >,
{
    #[inline]
    fn get_column_ref(&self) -> &C::ColumnType {
        GetColumn::get_column_ref(self.nested_index())
    }

    #[inline]
    fn get_column(&self) -> C::ColumnType {
        GetColumn::get_column(self.nested_index())
    }
}

/// Trait providing a failable getter for a specific Diesel column.
pub trait MayGetColumn<C: ColumnTyped> {
    /// Get the reference of the specified column, returning `None` if not
    /// present.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use diesel_builders::{MayGetColumn, SetColumn};
    /// use schema::users;
    ///
    /// let mut staged = users::table::empty_new_values();
    /// SetColumn::<users::name>::set_column(&mut staged, "Ada".to_owned());
    /// SetColumn::<users::nickname>::set_column(&mut staged, None::<String>);
    ///
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column_ref(&staged), Some(&"Ada".to_owned()));
    /// // nickname is staged as null while age is absent
    /// assert_eq!(MayGetColumn::<users::nickname>::may_get_column_ref(&staged), Some(&None));
    /// assert_eq!(MayGetColumn::<users::age>::may_get_column_ref(&staged), None);
    /// # }
    /// ```
    fn may_get_column_ref(&self) -> Option<&C::ColumnType>;
    /// Get the value of the specified column, returning `None` if not present.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::MayGetColumn;
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user: User = users::table.find(1).first(&mut conn)?;
    /// let present: Option<User> = Some(user);
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&present), Some("Ada".to_owned()));
    ///
    /// let absent: Option<User> = None;
    /// assert_eq!(MayGetColumn::<users::name>::may_get_column(&absent), None);
    /// # Ok(())
    /// # }
    /// ```
    fn may_get_column(&self) -> Option<C::ColumnType> {
        self.may_get_column_ref().cloned()
    }
}

impl<T, C> MayGetColumn<C> for Option<T>
where
    C: ColumnTyped,
    T: GetColumn<C>,
{
    #[inline]
    fn may_get_column_ref(&self) -> Option<&C::ColumnType> {
        Some(self.as_ref()?.get_column_ref())
    }
}

/// Extension trait for [`GetColumn`] that allows specifying the column at the
/// method level.
pub trait GetColumnExt {
    /// Get a reference to the specified column.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let ada: User = users::table.find(1).first(&mut conn)?;
    /// let grace: User = users::table.find(2).first(&mut conn)?;
    ///
    /// assert_eq!(ada.get_column_ref::<users::name>(), &"Ada".to_owned());
    /// assert_eq!(ada.get_column_ref::<users::nickname>(), &Some("Ace".to_owned()));
    /// assert_eq!(grace.get_column_ref::<users::nickname>(), &None);
    /// # Ok(())
    /// # }
    /// ```
    fn get_column_ref<Column>(&self) -> &Column::ColumnType
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column_ref(self)
    }

    /// Get the owned value of the specified column.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let ada: User = users::table.find(1).first(&mut conn)?;
    ///
    /// assert_eq!(ada.get_column::<users::id>(), 1);
    /// assert_eq!(ada.get_column::<users::age>(), 20);
    /// assert_eq!(ada.get_column::<users::nickname>(), Some("Ace".to_owned()));
    /// # Ok(())
    /// # }
    /// ```
    fn get_column<Column>(&self) -> Column::ColumnType
    where
        Column: TypedColumn,
        Self: GetColumn<Column>,
    {
        <Self as GetColumn<Column>>::get_column(self)
    }
}

impl<T> GetColumnExt for T {}

/// Extension trait for [`MayGetColumn`] that allows specifying the column at
/// the method level.
pub trait MayGetColumnExt {
    /// Get a reference to specified column, returning `None` if not present.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use schema::users;
    ///
    /// let mut staged = users::table::empty_new_values();
    /// staged.set_column_ref::<users::name>("Ada".to_owned());
    /// staged.set_column_ref::<users::nickname>("Ace".to_owned());
    ///
    /// assert_eq!(staged.may_get_column_ref::<users::name>(), Some(&"Ada".to_owned()));
    /// assert_eq!(staged.may_get_column_ref::<users::nickname>(), Some(&Some("Ace".to_owned())));
    /// assert_eq!(staged.may_get_column_ref::<users::age>(), None);
    /// # }
    /// ```
    fn may_get_column_ref<'a, Column>(&'a self) -> Option<&'a Column::ColumnType>
    where
        Column: TypedColumn,
        Column::Table: 'a,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column_ref(self)
    }

    /// Get the owned value of the specified column, returning `None` if not
    /// present.
    ///
    /// # Examples
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() {
    /// use schema::users;
    ///
    /// let mut staged = users::table::empty_new_values();
    /// staged.set_column_ref::<users::age>(20);
    ///
    /// assert_eq!(staged.may_get_column::<users::age>(), Some(20));
    /// assert_eq!(staged.may_get_column::<users::nickname>(), Some(None));
    /// # }
    /// ```
    fn may_get_column<Column>(&self) -> Option<Column::ColumnType>
    where
        Column: TypedColumn,
        Self: MayGetColumn<Column>,
    {
        <Self as MayGetColumn<Column>>::may_get_column(self)
    }
}

impl<T> MayGetColumnExt for T {}

mod blanket_impls;
pub mod dynamic;
pub use dynamic::TryGetDynamicColumn;

pub mod dynamic_multi;
pub use dynamic_multi::TryGetDynamicColumns;
