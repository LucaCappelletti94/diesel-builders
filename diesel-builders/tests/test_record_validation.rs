//! Required values take precedence over record validation during insertion.

mod shared;

use diesel_builders::{BuilderError, IncompleteBuilderError, prelude::*};

/// A record rejected by its final validator.
#[derive(Debug, PartialEq, thiserror::Error)]
#[error("Invalid record")]
pub struct RecordError;

/// A record with one required insertion column.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = explicit_records)]
#[table_model(record_error = RecordError)]
pub struct ExplicitRecord {
    /// The caller-assigned primary key.
    id: i32,
}

impl ValidateRecord<explicit_records::table> for <explicit_records::table as TableExt>::NewValues {
    fn validate_record(&self) -> Result<(), RecordError> {
        if self.may_get_column::<explicit_records::id>().is_some() {
            Ok(())
        } else {
            Err(RecordError)
        }
    }
}

/// An interval whose final values must be ordered.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = checked_intervals)]
#[table_model(surrogate_key, record_error = RecordError)]
pub struct CheckedInterval {
    /// Generated primary key.
    id: i32,
    /// Inclusive start.
    start: i32,
    /// Exclusive end.
    end: i32,
}

impl ValidateRecord<checked_intervals::table>
    for <checked_intervals::table as TableExt>::NewValues
{
    fn validate_record(&self) -> Result<(), RecordError> {
        match (
            self.may_get_column::<checked_intervals::start>(),
            self.may_get_column::<checked_intervals::end>(),
        ) {
            (Some(start), Some(end)) if start < end => Ok(()),
            _ => Err(RecordError),
        }
    }
}

#[test]
fn missing_single_column_precedes_record_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    diesel::sql_query("CREATE TABLE explicit_records (id INTEGER PRIMARY KEY NOT NULL)")
        .execute(&mut conn)?;
    let error = explicit_records::table::empty_builder().insert(&mut conn).unwrap_err();
    assert!(matches!(
        error,
        BuilderError::Incomplete(IncompleteBuilderError::MissingMandatoryField {
            table_name: "explicit_records",
            field_name: "id"
        })
    ));
    let count: i64 = explicit_records::table.count().get_result(&mut conn)?;
    assert_eq!(count, 0);
    let record = explicit_records::table::builder().id(42).insert(&mut conn)?;
    assert_eq!(record.id, 42);
    Ok(())
}

#[test]
fn missing_head_and_tail_precede_record_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = shared::establish_connection()?;
    diesel::sql_query("CREATE TABLE checked_intervals (id INTEGER PRIMARY KEY, start INTEGER NOT NULL, end INTEGER NOT NULL)").execute(&mut conn)?;
    for (builder, field_name) in [
        (checked_intervals::table::empty_builder().end(5), "start"),
        (checked_intervals::table::empty_builder().start(1), "end"),
    ] {
        let error = builder.insert(&mut conn).unwrap_err();
        assert!(
            matches!(error, BuilderError::Incomplete(IncompleteBuilderError::MissingMandatoryField { table_name: "checked_intervals", field_name: missing }) if missing == field_name)
        );
    }
    let error = checked_intervals::table::builder().start(5).end(1).insert(&mut conn).unwrap_err();
    assert!(matches!(error, BuilderError::Validation(_)));
    let count: i64 = checked_intervals::table.count().get_result(&mut conn)?;
    assert_eq!(count, 0);
    let record = checked_intervals::table::builder().start(1).end(5).insert(&mut conn)?;
    assert_eq!((record.start, record.end), (1, 5));
    let count: i64 = checked_intervals::table.count().get_result(&mut conn)?;
    assert_eq!(count, 1);
    Ok(())
}
