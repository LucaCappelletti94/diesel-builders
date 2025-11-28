//! Test for RustSqlType trait implementations

use diesel_additions::RustSqlType;

#[test]
fn test_rust_sql_type_mappings() {
    // Verify RustSqlType implementations for numeric types
    type IntegerType = <diesel::sql_types::Integer as RustSqlType>::Type;
    type SmallIntType = <diesel::sql_types::SmallInt as RustSqlType>::Type;
    type BigIntType = <diesel::sql_types::BigInt as RustSqlType>::Type;
    type FloatType = <diesel::sql_types::Float as RustSqlType>::Type;
    type DoubleType = <diesel::sql_types::Double as RustSqlType>::Type;
    type TinyIntType = <diesel::sql_types::TinyInt as RustSqlType>::Type;

    let _int: IntegerType = 42;
    let _small: SmallIntType = 25;
    let _big: BigIntType = 1000;
    let _float: FloatType = 98.5;
    let _double: DoubleType = 4.5;
    let _tiny: TinyIntType = 10;

    // Verify text and binary types
    type TextType = <diesel::sql_types::Text as RustSqlType>::Type;
    type BinaryType = <diesel::sql_types::Binary as RustSqlType>::Type;

    let _text: TextType = "test".to_string();
    let _binary: BinaryType = vec![1, 2, 3];

    // Verify boolean
    type BoolType = <diesel::sql_types::Bool as RustSqlType>::Type;
    let _bool: BoolType = true;
}

#[test]
fn test_nullable_types() {
    type NullableInteger =
        <diesel::sql_types::Nullable<diesel::sql_types::Integer> as RustSqlType>::Type;
    type NullableText = <diesel::sql_types::Nullable<diesel::sql_types::Text> as RustSqlType>::Type;

    let _nullable_int: NullableInteger = Some(42);
    let _nullable_text: NullableText = Some("test".to_string());
    let _null_int: NullableInteger = None;
    let _null_text: NullableText = None;
}

#[cfg(feature = "chrono")]
#[test]
fn test_date_time_types() {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    type DateType = <diesel::sql_types::Date as RustSqlType>::Type;
    type TimeType = <diesel::sql_types::Time as RustSqlType>::Type;
    type TimestampType = <diesel::sql_types::Timestamp as RustSqlType>::Type;

    let _date: DateType = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let _time: TimeType = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
    let _timestamp: TimestampType = NaiveDateTime::new(_date, _time);
}
