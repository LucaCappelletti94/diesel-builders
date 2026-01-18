//! Helper functions for creating builder-related errors.

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

impl<E: std::error::Error + 'static> std::fmt::Display for BuilderError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuilderError::Diesel(e) => write!(f, "Diesel error: {e}"),
            BuilderError::Incomplete(e) => write!(f, "{e}"),
            BuilderError::Validation(e) => write!(f, "Validation error: {e}"),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for BuilderError<E> {
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

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, thiserror::Error)]
/// Specific error indicating that not all mandatory triangular builder fields
/// have been set.
pub enum IncompleteBuilderError {
    #[error("Missing mandatory triangular builder field: `{table_name}.{field_name}`")]
    /// Not all mandatory associated builders have been set.
    MissingMandatoryTriangularField {
        /// The table of the missing column.
        table_name: &'static str,
        /// The name of the missing column.
        field_name: &'static str,
    },
    #[error("Missing mandatory field: `{table_name}.{field_name}`")]
    /// A field required for insertion is missing.
    MissingMandatoryField {
        /// The table of the missing column.
        table_name: &'static str,
        /// The name of the missing field.
        field_name: &'static str,
    },
}

/// Specific error indicating that a dynamic setting operation
/// has failed due to an incompatible/unknown column.
#[derive(Debug, thiserror::Error)]
pub enum DynamicColumnError {
    #[error("Unknown column: `{table_name}.{column_name}`")]
    /// The specified column is not part of the table.
    UnknownColumn {
        /// The table of the unknown column.
        table_name: &'static str,
        /// The name of the unknown column.
        column_name: &'static str,
    },
    #[error("Validation error: {0}")]
    /// Validation error when setting the column.
    Validation(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// A specialized `Result` type for builder operations.
pub type BuilderResult<T, E> = Result<T, BuilderError<E>>;

impl DatabaseErrorInformation for IncompleteBuilderError {
    fn message(&self) -> &str {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularField { .. } => {
                "Missing mandatory triangular builder field"
            }
            IncompleteBuilderError::MissingMandatoryField { .. } => "Missing mandatory field",
        }
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularField { table_name, .. }
            | IncompleteBuilderError::MissingMandatoryField { table_name, .. } => Some(table_name),
        }
    }

    fn column_name(&self) -> Option<&str> {
        match self {
            IncompleteBuilderError::MissingMandatoryTriangularField { field_name, .. }
            | IncompleteBuilderError::MissingMandatoryField { field_name, .. } => Some(field_name),
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
