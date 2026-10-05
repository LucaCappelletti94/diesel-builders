//! Submodule providing the `GetModel` trait.

use diesel::associations::HasTable;
use tuplities::prelude::{NestedTupleIndex, NestedTuplePopBack};

use crate::{
    AncestorOfIndex, DescendantOf, DescendantWithSelf, HasTableExt, NestedModel, NestedTables,
    TableExt,
};

/// Trait providing a getter for a specific table model.
pub trait GetModel<T: TableExt> {
    /// Get the value of the specified model.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetModel;
    /// use schema::{Profile, User, profiles, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = User::find(&1, &mut conn)?;
    /// let profile = Profile::find(&1, &mut conn)?;
    /// let nested = (user, (profile,));
    /// let ancestor: &User = GetModel::<users::table>::get_model_ref(&nested);
    /// assert_eq!(ancestor.id, 1);
    /// assert_eq!(ancestor.name, "Ada");
    /// let descendant: &Profile = GetModel::<profiles::table>::get_model_ref(&nested);
    /// assert_eq!(descendant.display_name, "Ada");
    /// assert_eq!(descendant.visits, 3);
    /// # Ok(())
    /// # }
    /// ```
    fn get_model_ref(&self) -> &T::Model;
    /// Get the owned value of the specified model.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetModel;
    /// use schema::{Profile, User, profiles};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = User::find(&1, &mut conn)?;
    /// let profile = Profile::find(&1, &mut conn)?;
    /// let nested = (user, (profile,));
    /// let mut owned: Profile = GetModel::<profiles::table>::get_model(&nested);
    /// assert_eq!(owned, nested.1.0);
    /// owned.visits = 99;
    /// assert_ne!(owned, nested.1.0);
    /// # Ok(())
    /// # }
    /// ```
    fn get_model(&self) -> T::Model {
        self.get_model_ref().clone()
    }
}

/// Trait providing a getter for the nested model of a specific table.
pub trait GetNestedModel<T: DescendantWithSelf> {
    /// Get the nested model associated with the specified table.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use diesel_builders::GetNestedModel;
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let (parent, (child,)): (Parent, (Child,)) = children::table::builder()
    ///     .mandatory(sides::table::builder())
    ///     .discretionary(sides::table::builder())
    ///     .child_label("short")
    ///     .insert_nested(&mut conn)?;
    /// let nested = (parent, (child,));
    /// let mut extracted = GetNestedModel::<children::table>::get_nested_model(&nested);
    /// assert_eq!(extracted, nested);
    /// extracted.1.0.child_label = "changed".into();
    /// assert_eq!(nested.1.0.child_label, "short");
    /// # Ok(())
    /// # }
    /// ```
    fn get_nested_model(&self) -> NestedModel<T>;
}

impl<T> GetModel<T> for (T::Model,)
where
    T: TableExt,
{
    #[inline]
    fn get_model_ref(&self) -> &T::Model {
        &self.0
    }

    #[inline]
    fn get_model(&self) -> T::Model {
        self.0.clone()
    }
}

impl<T> GetNestedModel<T> for (T::Model,)
where
    T: DescendantWithSelf<NestedAncestorsWithSelf = (T,)>,
{
    #[inline]
    fn get_nested_model(&self) -> NestedModel<T> {
        self.clone()
    }
}

impl<Head, Tail, T> GetModel<T> for (Head, Tail)
where
    T: TableExt + AncestorOfIndex<<Tail::Back as HasTable>::Table>,
    Tail: NestedTuplePopBack<Back: HasTableExt<Table: DescendantOf<T>>>,
    (Head, Tail): NestedTupleIndex<
            <T as AncestorOfIndex<<Tail::Back as HasTable>::Table>>::Idx,
            Element = T::Model,
        >,
{
    #[inline]
    fn get_model_ref(&self) -> &T::Model {
        self.nested_index()
    }

    #[inline]
    fn get_model(&self) -> T::Model {
        self.nested_index().clone()
    }
}

/// Helper trait to extract the nested model of a table.
trait ExtractNestedModels<T: NestedTables> {
    /// Extract the nested model.
    fn extract(&self) -> T::NestedModels;
}

impl<S, T> ExtractNestedModels<(T,)> for S
where
    T: DescendantWithSelf,
    S: GetModel<T>,
{
    fn extract(&self) -> (T::Model,) {
        (GetModel::<T>::get_model(self),)
    }
}

impl<S, Head, Tail> ExtractNestedModels<(Head, Tail)> for S
where
    Head: DescendantWithSelf,
    Tail: NestedTables,
    (Head, Tail): NestedTables<NestedModels = (Head::Model, Tail::NestedModels)>,
    S: GetModel<Head> + ExtractNestedModels<Tail>,
{
    fn extract(&self) -> (Head::Model, Tail::NestedModels) {
        (GetModel::<Head>::get_model(self), ExtractNestedModels::<Tail>::extract(self))
    }
}

impl<Head, Tail, T> GetNestedModel<T> for (Head, Tail)
where
    T: DescendantWithSelf + AncestorOfIndex<<Tail::Back as HasTable>::Table>,
    Tail: NestedTuplePopBack<Back: HasTableExt<Table: DescendantOf<T>>>,
    (Head, Tail): ExtractNestedModels<T::NestedAncestorsWithSelf>,
{
    #[inline]
    fn get_nested_model(&self) -> NestedModel<T> {
        ExtractNestedModels::<T::NestedAncestorsWithSelf>::extract(self)
    }
}

/// Alternative version of the `GetModel` which moved the
/// table type parameter to the methods.
pub trait GetModelExt {
    /// Get the value of the specified model.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{Profile, User, profiles, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = User::find(&1, &mut conn)?;
    /// let profile = Profile::find(&1, &mut conn)?;
    /// let nested = (user, (profile,));
    /// let ancestor: &User = nested.get_model_ref::<users::table>();
    /// assert_eq!(ancestor.name, "Ada");
    /// let descendant: &Profile = nested.get_model_ref::<profiles::table>();
    /// assert_eq!(descendant.visits, 3);
    /// # Ok(())
    /// # }
    /// ```
    fn get_model_ref<T>(&self) -> &T::Model
    where
        T: DescendantWithSelf,
        Self: GetModel<T>,
    {
        GetModel::get_model_ref(self)
    }

    /// Get the owned value of the specified model.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{Profile, User, users};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = User::find(&1, &mut conn)?;
    /// let profile = Profile::find(&1, &mut conn)?;
    /// let nested = (user, (profile,));
    /// let mut owned: User = nested.get_model::<users::table>();
    /// assert_eq!(owned, nested.0);
    /// owned.nickname = None;
    /// assert_eq!(nested.0.nickname.as_deref(), Some("Ace"));
    /// # Ok(())
    /// # }
    /// ```
    fn get_model<T>(&self) -> T::Model
    where
        T: DescendantWithSelf<Model: Clone>,
        Self: GetModel<T>,
    {
        GetModel::get_model(self)
    }
}

impl<T> GetModelExt for T {}

/// Alternative version of the `GetNestedModel` which moved the
/// table type parameter to the methods.
pub trait GetNestedModelExt {
    /// Get the nested model associated with the specified table.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::{Profile, User, profiles};
    ///
    /// let mut conn = connection_with_data()?;
    /// let user = User::find(&1, &mut conn)?;
    /// let profile = Profile::find(&1, &mut conn)?;
    /// let nested = (user, (profile,));
    /// let (extracted_user, (extracted_profile,)) = nested.get_nested_model::<profiles::table>();
    /// assert_eq!(extracted_user.name, "Ada");
    /// assert_eq!(extracted_user.nickname, Some("Ace".into()));
    /// assert_eq!(extracted_profile.display_name, "Ada");
    /// assert_eq!(extracted_profile.visits, 3);
    /// # Ok(())
    /// # }
    /// ```
    fn get_nested_model<T>(&self) -> NestedModel<T>
    where
        T: DescendantWithSelf,
        Self: GetNestedModel<T>,
    {
        GetNestedModel::get_nested_model(self)
    }
}

impl<T> GetNestedModelExt for T {}
