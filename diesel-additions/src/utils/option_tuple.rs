//! Submodule defining a trait which, for any tuple tuple (T1, T2, ...), defines
//! an associated type which is the `(Option<T1>, Option<T2>, ...)` tuple.

use crate::DefaultTuple;

/// A trait for converting a tuple type into its corresponding option tuple
/// type.
pub trait OptionTuple {
    /// The associated option tuple type.
    type Output: DefaultTuple;

    /// Convert the tuple into its optional variant.
    fn into_option(self) -> Self::Output;
}

impl OptionTuple for () {
    type Output = ();

    fn into_option(self) -> Self::Output {
        ()
    }
}

macro_rules! impl_option_tuple {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> OptionTuple for ($head,)
		{
			type Output = (Option<$head>,);

			fn into_option(self) -> Self::Output {
				(Some(self.0),)
			}
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> OptionTuple for ($head, $($tail),+)
		{
			type Output = (
				Option<$head>,
				$(Option<$tail>),+
			);

			#[allow(non_snake_case)]
			fn into_option(self) -> Self::Output {
				let ($head, $($tail),+) = self;
				(Some($head), $(Some($tail)),+)
			}
		}

		impl_option_tuple!($($tail),+);
	};
}

generate_tuple_impls!(impl_option_tuple);
