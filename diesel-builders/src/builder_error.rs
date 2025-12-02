//! Helper functions for creating builder-specific Diesel errors.
//!
//! This module provides convenience functions for creating builder-related
//! errors using Diesel's `QueryBuilderError` variant, plus trait implementations
//! to make diesel::result::Error work with builder operations.

/// Wrap a validation error as a Diesel query builder error.
pub fn validation_error(
    error: impl core::error::Error + Send + Sync + 'static,
) -> diesel::result::Error {
    diesel::result::Error::QueryBuilderError(Box::new(error))
}

/// Error type for incomplete builder operations.
#[derive(Debug)]
pub enum IncompleteBuilderError {
    /// Missing mandatory triangular builder fields.
    MissingMandatoryTriangularFields,
}

impl std::fmt::Display for IncompleteBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularFields => {
                write!(f, "Not all mandatory associated builders have been set")
            }
        }
    }
}

impl core::error::Error for IncompleteBuilderError {}

impl From<IncompleteBuilderError> for diesel::result::Error {
    fn from(error: IncompleteBuilderError) -> Self {
        diesel::result::Error::QueryBuilderError(Box::new(error))
    }
}
