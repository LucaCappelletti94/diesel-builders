//! Submodule providing the `GetColumns` trait.

use crate::{
    Columns, GetColumn, HomogeneousColumns, MayGetColumn, SetInsertableTableModelColumn,
    TypedColumn,
};

/// Marker trait indicating a builder can get multiple columns.
pub trait GetColumns<CS: Columns> {}

/// Marker trait indicating a builder which may get multiple columns.
pub trait MayGetColumns<CS: Columns> {}

/// Marker trait indicating a builder can set multiple columns.
pub trait SetColumns<CS: Columns> {}

/// Marker trait indicating a builder can set multiple homogeneous columns.
pub trait SetInsertableTableModelHomogeneousColumn<CS: HomogeneousColumns> {
    /// Set the values of the specified columns.
    fn set(&mut self, value: &<CS as HomogeneousColumns>::Type);
}

/// Marker trait indicating a builder can try to set multiple columns.
pub trait TrySetColumns<CS: Columns> {}

/// Marker trait indicating a builder can try to set multiple homogeneous
/// columns.
pub trait TrySetInsertableTableModelHomogeneousColumn<CS: HomogeneousColumns> {
    /// Attempt to set the values of the specified columns.
    fn try_set(&mut self, value: &<CS as HomogeneousColumns>::Type) -> anyhow::Result<()>;
}

// Recursive macro that implements `Columns` for tuples of decreasing length.
// Call it with a list of type idents and it will generate impls for the full
// tuple, then the tuple without the first element, and so on, down to 1.
macro_rules! impl_get_columns {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<T, $head> GetColumns<($head,)> for T
		where
			T: GetColumn<$head>,
			$head: TypedColumn
		{
		}

		impl<T, $head> MayGetColumns<($head,)> for T
		where T: MayGetColumn<$head>, $head: TypedColumn
		{
		}

		impl<T, $head> SetColumns<($head,)> for T
		where T: crate::set_column::SetColumn<$head>, $head: TypedColumn
		{
		}

		impl<T, $head> SetInsertableTableModelHomogeneousColumn<($head,)> for T
		where T: SetInsertableTableModelColumn<$head>, $head: TypedColumn
		{
			fn set(&mut self, value: &<$head as TypedColumn>::Type) {
				self.set(value);
			}
		}

		impl<T, $head> TrySetColumns<($head,)> for T
		where T: crate::set_column::TrySetColumn<$head>, $head: TypedColumn
		{
		}

		impl<T, $head> TrySetInsertableTableModelHomogeneousColumn<($head,)> for T
		where T: crate::set_column::TrySetColumn<$head>, $head: TypedColumn
		{
			fn try_set(&mut self, value: &<$head as TypedColumn>::Type) -> anyhow::Result<()> {
				self.try_set(value)
			}
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<T, $head, $($tail),+> GetColumns<($head, $($tail),+)> for T
		where T: GetColumn<$head>, $(T: GetColumn<$tail>),+,
				$head: TypedColumn, $($tail: TypedColumn),+
		{
		}

		impl<T, $head, $($tail),+> MayGetColumns<($head, $($tail),+)> for T
		where T: MayGetColumn<$head>, $(T: MayGetColumn<$tail>),+,
				$head: TypedColumn, $($tail: TypedColumn),+
		{
		}

		impl<T, $head, $($tail),+> SetColumns<($head, $($tail),+)> for T
		where T: crate::set_column::SetColumn<$head>, $(T: crate::set_column::SetColumn<$tail>),+,
				$head: TypedColumn, $($tail: TypedColumn),+
		{
		}

		impl<T, $head, $($tail),+> SetInsertableTableModelHomogeneousColumn<($head, $($tail),+)> for T
		where
			T: SetInsertableTableModelColumn<$head>,
			$(T: SetInsertableTableModelColumn<$tail>),+,
			$head: TypedColumn,
			$($tail: TypedColumn<Type=$head::Type>),+
		{
			fn set(&mut self, value: &<$head as TypedColumn>::Type) {
				<T as SetInsertableTableModelColumn<$head>>::set(self, value);
				$(
					<T as SetInsertableTableModelColumn<$tail>>::set(self, value);
				)+
			}
		}

		impl<T, $head, $($tail),+> TrySetColumns<($head, $($tail),+)> for T
		where
			T: crate::set_column::TrySetColumn<$head>,
			$(T: crate::set_column::TrySetColumn<$tail>),+,
			$head: TypedColumn,
			$($tail: TypedColumn<Type=$head::Type>),+
		{
		}

		impl<T, $head, $($tail),+> TrySetInsertableTableModelHomogeneousColumn<($head, $($tail),+)> for T
		where
			T: crate::set_column::TrySetColumn<$head>,
			$(T: crate::set_column::TrySetColumn<$tail>),+,
			$head: TypedColumn,
			$($tail: TypedColumn<Type=$head::Type>),+
		{
			fn try_set(&mut self, value: &<$head as TypedColumn>::Type) -> anyhow::Result<()> {
				<T as crate::set_column::TrySetColumn<$head>>::try_set(self, value)?;
				$(
					<T as crate::set_column::TrySetColumn<$tail>>::try_set(self, value)?;
				)+
				Ok(())
			}
		}

		impl_get_columns!($($tail),+);
	};
}

generate_tuple_impls!(impl_get_columns);
