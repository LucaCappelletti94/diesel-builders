//! Submodule defining and implementing the `ExtendTuple` trait.

/// A trait for extending one tuple with another.
pub trait ExtendTuple<Other> {
    /// The resulting tuple type after extension.
    type Output;
}

impl<T> ExtendTuple<(T,)> for () {
    type Output = (T,);
}

macro_rules! impl_extend_tuple {
	($($left:ident),+; $($right:ident),+) => {
		impl<$($left,)+ $($right,)+> ExtendTuple<($($right,)+)> for ($($left,)+) {
			type Output = ($($left,)+ $($right,)+);
		}
	};
}

impl_extend_tuple!(A; B);
impl_extend_tuple!(A, B; C);
impl_extend_tuple!(A, B, C; D);
impl_extend_tuple!(A, B, C, D; E);
impl_extend_tuple!(A, B, C, D, E; F);
impl_extend_tuple!(A, B, C, D, E, F; G);
impl_extend_tuple!(A, B, C, D, E, F, G; H);
