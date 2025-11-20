//! Submodule defining a trait which, for any tuple tuple (T1, T2, ...), defines
//! an associated type which is the `(Option<T1>, Option<T2>, ...)` tuple.

/// A trait for converting a tuple type into its corresponding option tuple
/// type.
pub trait OptionTuple {
    /// The associated option tuple type.
    type Output;
}

impl OptionTuple for () {
    type Output = ();
}

macro_rules! impl_option_tuple {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> OptionTuple for ($head,)
		{
			type Output = (Option<$head>,);
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
		}

		impl_option_tuple!($($tail),+);
	};
}

// Generate implementations for tuples up to 32 elements.
impl_option_tuple!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30, T31, T32
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_tuple_two() {
        let _: <(i32, u8) as OptionTuple>::Output = (Some(0i32), Some(0u8));
    }

    #[test]
    fn option_tuple_three() {
        let _: <(i32, u8, bool) as OptionTuple>::Output = (Some(0i32), Some(0u8), Some(false));
    }
}
