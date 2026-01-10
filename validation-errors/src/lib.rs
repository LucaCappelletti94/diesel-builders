//! Crate providing common validation errors.

use core::convert::Infallible;

use diesel::result::DatabaseErrorInformation;

#[derive(Debug, thiserror::Error)]
/// Enumeration of errors that can occur during validation.
pub enum ValidationErrorKind {
    /// The provided entries should be distinct.
    #[error("Fields `{0}` and `{1}` must be distinct")]
    MustBeDistinct(&'static str, &'static str),
    /// The provided left entry must be strictly smaller than the right entry.
    #[error("Field `{0}` must be strictly smaller than field `{1}`")]
    MustBeStrictlySmallerThan(&'static str, &'static str),
    /// The provided left entry must be smaller than the right entry.
    #[error("Field `{0}` must be smaller than or equal to field `{1}`")]
    MustBeSmallerThan(&'static str, &'static str),
    /// The provided left entry must be strictly greater than the right entry.
    #[error("Field `{0}` must be strictly greater than field `{1}`")]
    MustBeStrictlyGreaterThan(&'static str, &'static str),
    /// The provided left entry must be greater than the right entry.
    #[error("Field `{0}` must be greater than or equal to field `{1}`")]
    MustBeGreaterThan(&'static str, &'static str),
    /// The provided text is empty.
    #[error("Field `{0}` must not be empty")]
    MustNotBeEmpty(&'static str),
    /// The scalar is not strictly greater than the expected amount.
    #[error("Field `{0}` must be strictly smaller than {1}")]
    MustBeStrictlySmallerThanScalar(&'static str, f64),
    /// The scalar is not smaller than the expected amount.
    #[error("Field `{0}` must be smaller than or equal to {1}")]
    MustBeSmallerThanScalar(&'static str, f64),
    /// The scalar is not strictly greater than the expected amount.
    #[error("Field `{0}` must be strictly greater than {1}")]
    MustBeStrictlyGreaterThanScalar(&'static str, f64),
    /// The scalar is not greater than the expected amount.
    #[error("Field `{0}` must be greater than or equal to {1}")]
    MustBeGreaterThanScalar(&'static str, f64),
    /// Some third-party validation error.
    #[error("Fields {fields:?}: {error}")]
    Generic {
        /// The fields involved in the error.
        fields: Vec<&'static str>,
        #[source]
        /// The underlying error.
        error: Box<dyn core::error::Error + Send + Sync>,
    },
}

impl AsRef<str> for ValidationErrorKind {
    fn as_ref(&self) -> &str {
        // For simplicity, return a static string for each variant
        match self {
            ValidationErrorKind::MustBeDistinct(_, _) => "Fields must be distinct",
            ValidationErrorKind::MustBeStrictlySmallerThan(_, _) => {
                "Field must be strictly smaller than another"
            }
            ValidationErrorKind::MustBeSmallerThan(_, _) => {
                "Field must be smaller than or equal to another"
            }
            ValidationErrorKind::MustBeStrictlyGreaterThan(_, _) => {
                "Field must be strictly greater than another"
            }
            ValidationErrorKind::MustBeGreaterThan(_, _) => {
                "Field must be greater than or equal to another"
            }
            ValidationErrorKind::MustNotBeEmpty(_) => "Field must not be empty",
            ValidationErrorKind::MustBeStrictlySmallerThanScalar(_, _) => {
                "Field must be strictly smaller than value"
            }
            ValidationErrorKind::MustBeSmallerThanScalar(_, _) => {
                "Field must be smaller than or equal to value"
            }
            ValidationErrorKind::MustBeStrictlyGreaterThanScalar(_, _) => {
                "Field must be strictly greater than value"
            }
            ValidationErrorKind::MustBeGreaterThanScalar(_, _) => {
                "Field must be greater than or equal to value"
            }
            ValidationErrorKind::Generic { .. } => "Generic validation error",
        }
    }
}

impl From<Infallible> for ValidationError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

impl From<ValidationError> for diesel::result::Error {
    fn from(error: ValidationError) -> Self {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(error),
        )
    }
}

impl DatabaseErrorInformation for ValidationError {
    fn message(&self) -> &str {
        // Use the AsRef<str> implementation of the kind
        self.kind.as_ref()
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table_name(&self) -> Option<&str> {
        Some(self.table)
    }

    fn column_name(&self) -> Option<&str> {
        match &self.kind {
            ValidationErrorKind::MustNotBeEmpty(field)
            | ValidationErrorKind::MustBeStrictlySmallerThanScalar(field, _)
            | ValidationErrorKind::MustBeSmallerThanScalar(field, _)
            | ValidationErrorKind::MustBeStrictlyGreaterThanScalar(field, _)
            | ValidationErrorKind::MustBeGreaterThanScalar(field, _) => Some(*field),
            ValidationErrorKind::MustBeDistinct(field1, _)
            | ValidationErrorKind::MustBeStrictlySmallerThan(field1, _)
            | ValidationErrorKind::MustBeSmallerThan(field1, _)
            | ValidationErrorKind::MustBeStrictlyGreaterThan(field1, _)
            | ValidationErrorKind::MustBeGreaterThan(field1, _) => Some(*field1),
            ValidationErrorKind::Generic { fields, .. } => fields.first().copied(),
        }
    }

    fn constraint_name(&self) -> Option<&str> {
        None
    }

    fn statement_position(&self) -> Option<i32> {
        None
    }
}

#[derive(Debug, thiserror::Error)]
/// Enumeration of errors that can occur during validation.
#[error("Table `{table}`: {kind}")]
pub struct ValidationError {
    /// The table where the error occurred.
    table: &'static str,
    /// The kind of validation error.
    #[source]
    kind: ValidationErrorKind,
}

impl ValidationError {
    /// Returns the underlying kind of validation error.
    #[must_use]
    pub fn kind(&self) -> &ValidationErrorKind {
        &self.kind
    }

    /// Returns the table where the error occurred.
    #[must_use]
    pub fn table(&self) -> &'static str {
        self.table
    }

    /// Creates a new validation error for an empty field.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `field` - The name of the field that is empty.
    ///
    /// # Returns
    ///
    /// A `ValidationError` indicating that the specified field is empty.
    #[must_use]
    pub fn empty(table: &'static str, field: &'static str) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustNotBeEmpty(field),
        }
    }

    /// Creates a new validation error for two fields that must not be equal.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `left_field` - The name of the first field.
    /// * `right_field` - The name of the second field.
    #[must_use]
    pub fn equal(table: &'static str, left_field: &'static str, right_field: &'static str) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeDistinct(left_field, right_field),
        }
    }

    /// Creates a new validation error for a field who should be smaller than or
    /// equal to another field.
    ///
    /// # Arguments
    ///
    /// * `smaller_field` - The name of the field that should be smaller.
    /// * `greater_field` - The name of the field that should be greater.
    #[must_use]
    pub fn smaller_than(
        table: &'static str,
        smaller_field: &'static str,
        greater_field: &'static str,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeSmallerThan(smaller_field, greater_field),
        }
    }

    /// Creates a new validation error for a field who should be smaller than or
    /// equal to a provided value.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `field` - The name of the field that should be smaller.
    /// * `value` - The value that the field should be smaller than or equal to.
    #[must_use]
    pub fn smaller_than_value(table: &'static str, field: &'static str, value: f64) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeSmallerThanScalar(field, value),
        }
    }

    /// Creates a new validation error for a field who should be greater than a
    /// another field.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `greater_field` - The name of the field that should be greater.
    /// * `smaller_field` - The name of the field that should be smaller.
    #[must_use]
    pub fn greater_than(
        table: &'static str,
        greater_field: &'static str,
        smaller_field: &'static str,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeGreaterThan(greater_field, smaller_field),
        }
    }

    /// Creates a new validation error for a field who should be greater than or
    /// equal to a provided value.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `field` - The name of the field that should be greater.
    /// * `value` - The value that the field should be greater than or equal to.
    #[must_use]
    pub fn greater_than_value(table: &'static str, field: &'static str, value: f64) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeGreaterThanScalar(field, value),
        }
    }

    /// Creates a new validation error for a field who should be strictly
    /// smaller than another field.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `smaller_equal_field` - The name of the field that should be strictly
    ///   smaller than another field.
    /// * `greater_field` - The name of the field that should be greater.
    #[must_use]
    pub fn strictly_smaller_than(
        table: &'static str,
        smaller_equal_field: &'static str,
        greater_field: &'static str,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeStrictlySmallerThan(
                smaller_equal_field,
                greater_field,
            ),
        }
    }

    /// Creates a new validation error for a field who should be strictly
    /// smaller than a provided value.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `field` - The name of the field that should be strictly smaller.
    /// * `value` - The value that the field should be strictly smaller than.
    #[must_use]
    pub fn strictly_smaller_than_value(
        table: &'static str,
        field: &'static str,
        value: f64,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeStrictlySmallerThanScalar(field, value),
        }
    }

    /// Creates a new validation error for a field who should be strictly
    /// greater than another field.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `greater_equal_field` - The name of the field that should be strictly
    ///   greater than another field.
    /// * `smaller_field` - The name of the field that should be smaller.
    #[must_use]
    pub fn strictly_greater_than(
        table: &'static str,
        greater_equal_field: &'static str,
        smaller_field: &'static str,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeStrictlyGreaterThan(
                greater_equal_field,
                smaller_field,
            ),
        }
    }

    /// Creates a new validation error for a field who should be strictly
    /// greater than a provided value.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `field` - The name of the field that should be strictly greater.
    /// * `value` - The value that the field should be strictly greater than.
    #[must_use]
    pub fn strictly_greater_than_value(
        table: &'static str,
        field: &'static str,
        value: f64,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::MustBeStrictlyGreaterThanScalar(field, value),
        }
    }

    /// Creates a new generic validation error.
    ///
    /// # Arguments
    ///
    /// * `table` - The name of the table where the error occurred.
    /// * `fields` - The fields involved in the error.
    /// * `error` - The underlying error.
    #[must_use]
    pub fn generic(
        table: &'static str,
        fields: Vec<&'static str>,
        error: Box<dyn core::error::Error + Send + Sync>,
    ) -> Self {
        ValidationError {
            table,
            kind: ValidationErrorKind::Generic { fields, error },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::error::Error;

    // Dummy error for testing
    #[derive(Debug)]
    struct DummyError;

    impl core::fmt::Display for DummyError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "dummy error")
        }
    }

    impl core::error::Error for DummyError {}

    unsafe impl Send for DummyError {}
    unsafe impl Sync for DummyError {}

    #[test]
    fn test_validation_error_kind_display() {
        use core::fmt::Write;

        // Test MustBeDistinct
        let err = ValidationErrorKind::MustBeDistinct("field1", "field2");
        let mut s = String::new();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Fields `field1` and `field2` must be distinct");

        // Test MustBeStrictlySmallerThan
        let err = ValidationErrorKind::MustBeStrictlySmallerThan("a", "b");
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `a` must be strictly smaller than field `b`");

        // Test MustBeSmallerThan
        let err = ValidationErrorKind::MustBeSmallerThan("a", "b");
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `a` must be smaller than or equal to field `b`");

        // Test MustBeStrictlyGreaterThan
        let err = ValidationErrorKind::MustBeStrictlyGreaterThan("a", "b");
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `a` must be strictly greater than field `b`");

        // Test MustBeGreaterThan
        let err = ValidationErrorKind::MustBeGreaterThan("a", "b");
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `a` must be greater than or equal to field `b`");

        // Test MustNotBeEmpty
        let err = ValidationErrorKind::MustNotBeEmpty("field");
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `field` must not be empty");

        // Test MustBeStrictlySmallerThanScalar
        let err = ValidationErrorKind::MustBeStrictlySmallerThanScalar("field", 5.0);
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `field` must be strictly smaller than 5");

        // Test MustBeSmallerThanScalar
        let err = ValidationErrorKind::MustBeSmallerThanScalar("field", 5.0);
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `field` must be smaller than or equal to 5");

        // Test MustBeStrictlyGreaterThanScalar
        let err = ValidationErrorKind::MustBeStrictlyGreaterThanScalar("field", 5.0);
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `field` must be strictly greater than 5");

        // Test MustBeGreaterThanScalar
        let err = ValidationErrorKind::MustBeGreaterThanScalar("field", 5.0);
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Field `field` must be greater than or equal to 5");

        // Test Generic
        let dummy = DummyError;
        let err = ValidationErrorKind::Generic {
            fields: vec!["field1", "field2"],
            error: Box::new(dummy),
        };
        s.clear();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Fields [\"field1\", \"field2\"]: dummy error");
    }

    #[test]
    fn test_validation_error_kind_source() {
        // Non-generic should have no source
        let err = ValidationErrorKind::MustNotBeEmpty("field");
        assert!(err.source().is_none());

        // Generic should have source
        let dummy = DummyError;
        let err = ValidationErrorKind::Generic {
            fields: vec!["field"],
            error: Box::new(dummy),
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn test_validation_error_display() {
        use core::fmt::Write;

        let err = ValidationError::empty("table", "field");
        let mut s = String::new();
        write!(s, "{err}").unwrap();
        assert_eq!(s, "Table `table`: Field `field` must not be empty");
    }

    #[test]
    fn test_validation_error_source() {
        // Non-generic kind should have source from kind
        let err = ValidationError::empty("table", "field");
        assert!(err.source().is_some());

        // Generic kind should chain source
        let dummy = DummyError;
        let kind = ValidationErrorKind::Generic {
            fields: vec!["field"],
            error: Box::new(dummy),
        };
        let err = ValidationError {
            table: "table",
            kind,
        };
        assert!(err.source().is_some());
    }

    #[test]
    fn test_validation_error_methods() {
        let err = ValidationError::empty("mytable", "myfield");
        assert_eq!(err.table(), "mytable");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustNotBeEmpty("myfield")
        ));
    }

    #[test]
    fn test_validation_error_constructors() {
        // Test empty
        let err = ValidationError::empty("table", "field");
        assert_eq!(err.table(), "table");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustNotBeEmpty("field")
        ));

        // Test equal
        let err = ValidationError::equal("table", "field1", "field2");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustBeDistinct("field1", "field2")
        ));

        // Test smaller_than
        let err = ValidationError::smaller_than("table", "small", "big");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustBeSmallerThan("small", "big")
        ));

        // Test smaller_than_value
        let err = ValidationError::smaller_than_value("table", "field", 10.0);
        assert!(
            matches!(err.kind(), ValidationErrorKind::MustBeSmallerThanScalar("field", v) if (*v - 10.0).abs() < f64::EPSILON)
        );

        // Test greater_than
        let err = ValidationError::greater_than("table", "big", "small");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustBeGreaterThan("big", "small")
        ));

        // Test greater_than_value
        let err = ValidationError::greater_than_value("table", "field", 10.0);
        assert!(
            matches!(err.kind(), ValidationErrorKind::MustBeGreaterThanScalar("field", v) if (*v - 10.0).abs() < f64::EPSILON)
        );

        // Test strictly_smaller_than
        let err = ValidationError::strictly_smaller_than("table", "small", "big");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustBeStrictlySmallerThan("small", "big")
        ));

        // Test strictly_smaller_than_value
        let err = ValidationError::strictly_smaller_than_value("table", "field", 10.0);
        assert!(
            matches!(err.kind(), ValidationErrorKind::MustBeStrictlySmallerThanScalar("field", v) if (*v - 10.0).abs() < f64::EPSILON)
        );

        // Test strictly_greater_than
        let err = ValidationError::strictly_greater_than("table", "big", "small");
        assert!(matches!(
            err.kind(),
            ValidationErrorKind::MustBeStrictlyGreaterThan("big", "small")
        ));

        // Test strictly_greater_than_value
        let err = ValidationError::strictly_greater_than_value("table", "field", 10.0);
        assert!(
            matches!(err.kind(), ValidationErrorKind::MustBeStrictlyGreaterThanScalar("field", v) if (*v - 10.0).abs() < f64::EPSILON)
        );

        // Test generic
        let dummy = DummyError;
        let err = ValidationError::generic("table", vec!["field1", "field2"], Box::new(dummy));
        assert!(
            matches!(err.kind(), ValidationErrorKind::Generic { fields, .. } if fields == &["field1", "field2"])
        );
    }

    #[test]
    fn test_database_error_information() {
        let err = ValidationError::empty("mytable", "myfield");

        assert_eq!(err.message(), "Field must not be empty");
        assert_eq!(err.details(), None);
        assert_eq!(err.hint(), None);
        assert_eq!(err.table_name(), Some("mytable"));
        assert_eq!(err.column_name(), Some("myfield"));
        assert_eq!(err.constraint_name(), None);
        assert_eq!(err.statement_position(), None);

        // Test multi-field
        let err = ValidationError::equal("table", "field1", "field2");
        assert_eq!(err.column_name(), Some("field1"));

        // Test scalar field
        let err = ValidationError::smaller_than_value("table", "field", 10.0);
        assert_eq!(err.column_name(), Some("field"));

        // Test two-field comparison
        let err = ValidationError::smaller_than("table", "small", "big");
        assert_eq!(err.column_name(), Some("small"));

        // Test generic
        let dummy = DummyError;
        let err = ValidationError::generic("table", vec!["field1", "field2"], Box::new(dummy));
        assert_eq!(err.column_name(), Some("field1"));
    }

    #[test]
    fn test_from_infallible() {
        // Infallible can't be created, but the impl exists
        // This test just ensures the impl compiles
        let _ = |x: Infallible| ValidationError::from(x);
    }

    #[test]
    fn test_as_ref_str() {
        let err = ValidationErrorKind::MustNotBeEmpty("field");
        assert_eq!(err.as_ref(), "Field must not be empty");

        let err = ValidationErrorKind::MustBeDistinct("a", "b");
        assert_eq!(err.as_ref(), "Fields must be distinct");

        let err = ValidationErrorKind::MustBeStrictlySmallerThan("a", "b");
        assert_eq!(err.as_ref(), "Field must be strictly smaller than another");

        let err = ValidationErrorKind::MustBeSmallerThan("a", "b");
        assert_eq!(
            err.as_ref(),
            "Field must be smaller than or equal to another"
        );

        let err = ValidationErrorKind::MustBeStrictlyGreaterThan("a", "b");
        assert_eq!(err.as_ref(), "Field must be strictly greater than another");

        let err = ValidationErrorKind::MustBeGreaterThan("a", "b");
        assert_eq!(
            err.as_ref(),
            "Field must be greater than or equal to another"
        );

        let err = ValidationErrorKind::MustBeStrictlySmallerThanScalar("field", 1.0);
        assert_eq!(err.as_ref(), "Field must be strictly smaller than value");

        let err = ValidationErrorKind::MustBeSmallerThanScalar("field", 1.0);
        assert_eq!(err.as_ref(), "Field must be smaller than or equal to value");

        let err = ValidationErrorKind::MustBeStrictlyGreaterThanScalar("field", 1.0);
        assert_eq!(err.as_ref(), "Field must be strictly greater than value");

        let err = ValidationErrorKind::MustBeGreaterThanScalar("field", 1.0);
        assert_eq!(err.as_ref(), "Field must be greater than or equal to value");

        let dummy = DummyError;
        let err = ValidationErrorKind::Generic {
            fields: vec!["field"],
            error: Box::new(dummy),
        };
        assert_eq!(err.as_ref(), "Generic validation error");
    }

    #[test]
    fn test_from_diesel_error() {
        let validation_err = ValidationError::empty("table", "field");
        let diesel_err: diesel::result::Error = validation_err.into();

        assert!(matches!(
            diesel_err,
            diesel::result::Error::DatabaseError(_, _)
        ));

        if let diesel::result::Error::DatabaseError(kind, info) = diesel_err {
            assert_eq!(kind, diesel::result::DatabaseErrorKind::Unknown);
            // The info should be our ValidationError
            assert_eq!(info.message(), "Field must not be empty");
            assert_eq!(info.table_name(), Some("table"));
            assert_eq!(info.column_name(), Some("field"));
        }
    }
}
