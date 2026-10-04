//! Submodule providing the `BuildableTables` trait and its implementations.

use std::convert::Infallible;

use tuplities::prelude::{NestedTupleIndex, NestedTupleTryFrom};

use crate::{
    AncestorOfIndex, IncompleteBuilderError, TableBuilder, TableBuilderBundle,
    ancestors::DescendantWithSelf, builder_bundle::BundlableTableExt, construction::BuildDefaults,
};

/// A trait for Diesel tables that can be used to build insertable models for
/// themselves and their ancestors.
///
/// This trait provides the core functionality for creating builders that handle
/// complex table relationships including inheritance hierarchies and triangular
/// dependencies. It ensures that all required related records are created in
/// the correct order.
///
/// Extends [`BundlableTableExt`] and [`DescendantWithSelf`].
///
/// # Type Parameters
///
/// * `NestedAncestorBuilders`: A nested tuple of builder bundles for ancestor
///   tables
/// * `NestedCompletedAncestorBuilders`: The completed version of ancestor
///   builders ready for insertion
/// * `NestedDefaultBundles`: A nested tuple of staged default bundles for the
///   table and its ancestors, indexed like the ordinary builder bundles
pub trait BuildableTable: BundlableTableExt + DescendantWithSelf {
    /// The ancestor builders associated with this table.
    type NestedAncestorBuilders: NestedTupleIndex<<Self as AncestorOfIndex<Self>>::Idx, Element = TableBuilderBundle<Self>>;
    /// The completed ancestor builders associated with this table.
    type NestedCompletedAncestorBuilders: NestedTupleTryFrom<Self::NestedAncestorBuilders, IncompleteBuilderError>;
    /// The staged default bundles for the table and its ancestors.
    type NestedDefaultBundles: BuildDefaults<CheckedBundles = Self::NestedAncestorBuilders>
        + Default;
    /// The error type of checked default construction for this table, covering
    /// the declared defaults and schema default overrides it can perform
    /// checks on.
    type DefaultError: std::error::Error + Send + Sync + 'static;

    /// Returns a new checked builder for the current table, validating every
    /// declared default and default override before the builder is exposed.
    ///
    /// # Errors
    ///
    /// Returns the validation error of the first rejected default.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let user = users::table::try_builder()?.name("Ada").insert(&mut conn)?;
    /// assert_eq!(user.name(), "Ada");
    /// assert_eq!(*user.age(), 18);
    ///
    /// let rejected = invalid_defaults::table::try_builder();
    /// assert!(matches!(rejected, Err(ValidationError::NegativeValue)));
    /// # Ok(())
    /// # }
    /// ```
    fn try_builder() -> Result<TableBuilder<Self>, Self::DefaultError>;

    /// Returns a new checked builder for the current table with no declared
    /// defaults applied, so explicit values can be provided even when a
    /// declared default is invalid.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let record = invalid_defaults::table::empty_builder().try_value(2)?.insert(&mut conn)?;
    /// assert_eq!(record.value, 2);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    fn empty_builder() -> TableBuilder<Self>;

    /// Returns a new instance of a builder for the current table, carrying
    /// the declared defaults.
    ///
    /// This is the primary entry point for creating records with complex
    /// relationships. The builder will handle dependency ordering
    /// automatically. It is only available when checked default construction
    /// is infallible.
    ///
    /// ```
    /// # include!("doctest_setup.rs");
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use schema::*;
    ///
    /// let mut conn = connection()?;
    /// let parent = parents::table::builder().label("Root").insert(&mut conn)?;
    /// assert_eq!(parent.label, "Root");
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    #[must_use]
    fn builder() -> TableBuilder<Self>
    where
        Self::DefaultError: Into<Infallible>,
    {
        Self::try_builder().map_err(Into::<Infallible>::into).unwrap_or_else(|error| match error {})
    }
}
