//! Submodule defining a trait for creating default tuples.

/// A trait for creating default values for tuple types.
pub trait DefaultTuple {
    /// Create a default instance of the tuple.
    fn default_tuple() -> Self;
}

impl DefaultTuple for () {
    fn default_tuple() -> Self {
        ()
    }
}

macro_rules! impl_default_tuple {
	// Single-element tuple (must include trailing comma)
	($head:ident) => {
		impl<$head> DefaultTuple for ($head,)
		where
			$head: Default,
		{
			fn default_tuple() -> Self {
				($head::default(),)
			}
		}
	};

	// Multi-element tuple: implement for the full tuple, then recurse on the tail.
	($head:ident, $($tail:ident),+) => {
		impl<$head, $($tail),+> DefaultTuple for ($head, $($tail),+)
		where
			$head: Default,
			$($tail: Default),+
		{
			fn default_tuple() -> Self {
				($head::default(), $($tail::default()),+)
			}
		}

		impl_default_tuple!($($tail),+);
	};
}

generate_tuple_impls!(impl_default_tuple);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tuple_unit() {
        let result = <()>::default_tuple();
        assert_eq!(result, ());
    }

    #[test]
    fn default_tuple_single() {
        let result = <(i32,)>::default_tuple();
        assert_eq!(result, (0,));
    }

    #[test]
    fn default_tuple_two() {
        let result = <(i32, String)>::default_tuple();
        assert_eq!(result, (0, String::new()));
    }

    #[test]
    fn default_tuple_three() {
        let result = <(i32, String, bool)>::default_tuple();
        assert_eq!(result, (0, String::new(), false));
    }

    #[test]
    fn default_tuple_mixed_types() {
        let result = <(Vec<i32>, Option<u8>, String)>::default_tuple();
        assert_eq!(result, (vec![], None, String::new()));
    }
}
