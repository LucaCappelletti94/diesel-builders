//! Helper functions for creating builder-related errors.

use std::fmt::Display;

use diesel::result::DatabaseErrorInformation;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
/// Specific error indicating that not all mandatory triangular builder fields
/// have been set.
pub enum IncompleteBuilderError {
    /// Not all mandatory associated builders have been set.
    MissingMandatoryTriangularField(&'static str),
    /// A field required for insertion is missing.
    MissingMandatoryField(&'static str),
}

/// Specific error indicating that a dynamic setting operation
/// has failed due to an incompatible/unknown column.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum DynamicSetColumnError<E> {
    /// The specified column is not part of the table.
    UnknownColumn(&'static str),
    /// Validation error when setting the column.
    Validation(E),
}

impl<E: Display> Display for DynamicSetColumnError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DynamicSetColumnError::UnknownColumn(column_name) => {
                write!(f, "Unknown column: `{column_name}`")
            }
            DynamicSetColumnError::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for DynamicSetColumnError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DynamicSetColumnError::UnknownColumn(_) => None,
            DynamicSetColumnError::Validation(e) => Some(e),
        }
    }
}

/// A specialized `Result` type for builder operations.
pub type BuilderResult<T, E> = Result<T, BuilderError<E>>;

impl core::fmt::Display for IncompleteBuilderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularField(horizontal_key_name) => {
                write!(f, "Missing mandatory triangular builder field: `{horizontal_key_name}`")
            }
            IncompleteBuilderError::MissingMandatoryField(field_name) => {
                write!(f, "Missing mandatory field: `{field_name}`")
            }
        }
    }
}

impl<E: core::fmt::Display> core::fmt::Display for BuilderError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BuilderError::Diesel(e) => write!(f, "Diesel error: {e}"),
            BuilderError::Incomplete(e) => write!(f, "{e}"),
            BuilderError::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl core::error::Error for IncompleteBuilderError {}

impl<E: core::error::Error + 'static> core::error::Error for BuilderError<E> {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
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

impl DatabaseErrorInformation for IncompleteBuilderError {
    fn message(&self) -> &str {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularField(_) => {
                "Missing mandatory triangular builder field"
            }
            IncompleteBuilderError::MissingMandatoryField(_) => "Missing mandatory field",
        }
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        None
    }

    fn column_name(&self) -> Option<&str> {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularField(field_name)
            | IncompleteBuilderError::MissingMandatoryField(field_name) => Some(field_name),
        }
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }

    fn statement_position(&self) -> Option<i32> {
        None
    }
}

impl<E: DatabaseErrorInformation + Send + Sync + 'static> From<BuilderError<E>>
    for diesel::result::Error
{
    fn from(error: BuilderError<E>) -> Self {
        match error {
            BuilderError::Diesel(e) => e,
            BuilderError::Incomplete(e) => {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    Box::new(e),
                )
            }
            BuilderError::Validation(e) => {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::CheckViolation,
                    Box::new(e),
                )
            }
        }
    }
}
