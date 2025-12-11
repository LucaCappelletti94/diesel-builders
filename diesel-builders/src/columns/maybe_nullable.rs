//! Submodule defining the `MaybeNullable` trait.

/// Trait for columns that can be nullable or not.
pub trait MaybeNullable {
    /// The output type of the column.
    type Output;
    /// Returns the column as a nullable or non-nullable column.
    fn maybe_nullable(self) -> Self::Output;
}

impl MaybeNullable for () {
    type Output = ();
    fn maybe_nullable(self) -> Self::Output {}
}

impl<Head> MaybeNullable for (Head,)
where
    Head: MaybeNullable,
{
    type Output = (Head::Output,);
    fn maybe_nullable(self) -> Self::Output {
        (self.0.maybe_nullable(),)
    }
}

impl<Head, Tail> MaybeNullable for (Head, Tail)
where
    Head: MaybeNullable,
    Tail: MaybeNullable,
{
    type Output = (Head::Output, Tail::Output);
    fn maybe_nullable(self) -> Self::Output {
        (self.0.maybe_nullable(), self.1.maybe_nullable())
    }
}
