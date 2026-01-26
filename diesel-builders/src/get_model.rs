//! Submodule providing the `GetModel` trait.

use diesel::associations::HasTable;
use tuplities::prelude::{NestedTupleIndex, NestedTuplePopBack};

use crate::{AncestorOfIndex, DescendantOf, HasTableExt, TableExt};

/// Trait providing a getter for a specific table model.
pub trait GetModel<T: TableExt> {
    /// Get the value of the specified model.
    fn get_model_ref(&self) -> &T::Model;
    /// Get the owned value of the specified model.
    fn get_model(&self) -> T::Model
    where
        T::Model: Clone,
    {
        self.get_model_ref().clone()
    }
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
    fn get_model(&self) -> T::Model
    where
        T::Model: Clone,
    {
        self.0.clone()
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
    fn get_model(&self) -> T::Model
    where
        T::Model: Clone,
    {
        self.nested_index().clone()
    }
}

/// Alternative version of the `GetModel` which moved the
/// table type parameter to the methods.
pub trait GetModelExt {
    /// Get the value of the specified model.
    fn get_model_ref<T>(&self) -> &T::Model
    where
        T: TableExt,
        Self: GetModel<T>,
    {
        GetModel::get_model_ref(self)
    }

    /// Get the owned value of the specified model.
    fn get_model<T>(&self) -> T::Model
    where
        T: TableExt<Model: Clone>,
        Self: GetModel<T>,
    {
        GetModel::get_model(self)
    }
}

impl<T> GetModelExt for T {}
