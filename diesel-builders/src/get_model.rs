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
    fn get_model_ref(&self) -> &T::Model;
    /// Get the owned value of the specified model.
    fn get_model(&self) -> T::Model {
        self.get_model_ref().clone()
    }
}

/// Trait providing a getter for the nested model of a specific table.
pub trait GetNestedModel<T: DescendantWithSelf> {
    /// Get the nested model associated with the specified table.
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

impl<S> ExtractNestedModels<()> for S {
    fn extract(&self) {}
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
    fn get_model_ref<T>(&self) -> &T::Model
    where
        T: DescendantWithSelf,
        Self: GetModel<T>,
    {
        GetModel::get_model_ref(self)
    }

    /// Get the owned value of the specified model.
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
    fn get_nested_model<T>(&self) -> NestedModel<T>
    where
        T: DescendantWithSelf,
        Self: GetNestedModel<T>,
    {
        GetNestedModel::get_nested_model(self)
    }
}

impl<T> GetNestedModelExt for T {}
