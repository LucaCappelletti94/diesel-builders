//! Helper functions for creating builder-related errors.

/// Error type for incomplete builder operations.
#[derive(Debug)]
pub enum BuilderError<E> {
    /// A diesel error.
    Diesel(diesel::result::Error),
    /// Missing mandatory triangular builder fields.
    Incomplete(IncompleteBuilderError),
    /// Underlying validation error.
    Validation(E),
}

#[derive(Debug)]
/// Specific error indicating that not all mandatory triangular builder fields
/// have been set.
pub enum IncompleteBuilderError {
    /// Not all mandatory associated builders have been set.
    MissingMandatoryTriangularFields,
}

/// A specialized `Result` type for builder operations.
pub type BuilderResult<T, E> = Result<T, BuilderError<E>>;

impl std::fmt::Display for IncompleteBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularFields => {
                write!(f, "Not all mandatory associated builders have been set")
            }
        }
    }
}

impl<E: std::fmt::Display> std::fmt::Display for BuilderError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::Diesel(e) => write!(f, "Diesel error: {e}"),
            BuilderError::Incomplete(e) => write!(f, "Incomplete builder error: {e}"),
            BuilderError::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl core::error::Error for IncompleteBuilderError {}

impl<E: std::error::Error + 'static> core::error::Error for BuilderError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BuilderError::Diesel(e) => Some(e),
            BuilderError::Incomplete(e) => Some(e),
            BuilderError::Validation(e) => Some(e),
        }
    }
}

impl<E> From<diesel::result::Error> for BuilderError<E> {
    fn from(error: diesel::result::Error) -> Self {
        BuilderError::Diesel(error)
    }
}

impl<E> From<IncompleteBuilderError> for BuilderError<E> {
    fn from(error: IncompleteBuilderError) -> Self {
        BuilderError::Incomplete(error)
    }
}
