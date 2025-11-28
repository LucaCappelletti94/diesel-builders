//! Trait for mapping Diesel SQL type markers to their corresponding Rust types.

/// A trait for mapping Diesel SQL type markers to their corresponding Rust
/// types.
///
/// This trait is implemented for all standard Diesel SQL types, providing a way
/// to determine the Rust type that corresponds to a given SQL type marker.
pub trait RustSqlType {
    /// The Rust type corresponding to this SQL type marker.
    type Type;
}

// Numeric types
impl RustSqlType for diesel::sql_types::SmallInt {
    type Type = i16;
}

impl RustSqlType for diesel::sql_types::Integer {
    type Type = i32;
}

impl RustSqlType for diesel::sql_types::BigInt {
    type Type = i64;
}

impl RustSqlType for diesel::sql_types::Float {
    type Type = f32;
}

impl RustSqlType for diesel::sql_types::Double {
    type Type = f64;
}

// Text types
impl RustSqlType for diesel::sql_types::Text {
    type Type = String;
}

// Binary types
impl RustSqlType for diesel::sql_types::Binary {
    type Type = Vec<u8>;
}

// Boolean
impl RustSqlType for diesel::sql_types::Bool {
    type Type = bool;
}

// Date and time types
impl RustSqlType for diesel::sql_types::Date {
    type Type = chrono::NaiveDate;
}

impl RustSqlType for diesel::sql_types::Time {
    type Type = chrono::NaiveTime;
}

impl RustSqlType for diesel::sql_types::Timestamp {
    type Type = chrono::NaiveDateTime;
}

// Interval (PostgreSQL)
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Interval {
    type Type = std::time::Duration;
}

// JSON types
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Json {
    type Type = serde_json::Value;
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Jsonb {
    type Type = serde_json::Value;
}

// UUID (PostgreSQL)
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Uuid {
    type Type = uuid::Uuid;
}

// Array types (PostgreSQL)
#[cfg(feature = "postgres")]
impl<T: RustSqlType> RustSqlType for diesel::sql_types::Array<T> {
    type Type = Vec<T::Type>;
}

// Nullable types
impl<T: RustSqlType> RustSqlType for diesel::sql_types::Nullable<T> {
    type Type = Option<T::Type>;
}

// Additional numeric types
impl RustSqlType for diesel::sql_types::TinyInt {
    type Type = i8;
}

#[cfg(feature = "numeric")]
impl RustSqlType for diesel::sql_types::Numeric {
    type Type = bigdecimal::BigDecimal;
}

// MySQL specific types
#[cfg(feature = "mysql")]
impl RustSqlType for diesel::sql_types::Datetime {
    type Type = chrono::NaiveDateTime;
}

#[cfg(feature = "mysql")]
impl RustSqlType for diesel::sql_types::Unsigned<diesel::sql_types::TinyInt> {
    type Type = u8;
}

#[cfg(feature = "mysql")]
impl RustSqlType for diesel::sql_types::Unsigned<diesel::sql_types::SmallInt> {
    type Type = u16;
}

#[cfg(feature = "mysql")]
impl RustSqlType for diesel::sql_types::Unsigned<diesel::sql_types::Integer> {
    type Type = u32;
}

#[cfg(feature = "mysql")]
impl RustSqlType for diesel::sql_types::Unsigned<diesel::sql_types::BigInt> {
    type Type = u64;
}

// Timestamptz (PostgreSQL with timezone)
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Timestamptz {
    type Type = chrono::DateTime<chrono::Utc>;
}

// SQLite specific - all integers map to i32 by default
// but we already have Integer -> i32, so no additional needed

// Network address types (PostgreSQL)
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Inet {
    type Type = ipnetwork::IpNetwork;
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Cidr {
    type Type = ipnetwork::IpNetwork;
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::MacAddr {
    type Type = [u8; 6];
}

// Money type (PostgreSQL)
#[cfg(all(feature = "postgres", feature = "numeric"))]
impl RustSqlType for diesel::sql_types::Money {
    type Type = bigdecimal::BigDecimal;
}

// Range types (PostgreSQL)
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Range<diesel::sql_types::Integer> {
    type Type = (std::ops::Bound<i32>, std::ops::Bound<i32>);
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Range<diesel::sql_types::BigInt> {
    type Type = (std::ops::Bound<i64>, std::ops::Bound<i64>);
}

#[cfg(all(feature = "postgres", feature = "numeric"))]
impl RustSqlType for diesel::sql_types::Range<diesel::sql_types::Numeric> {
    type Type = (std::ops::Bound<bigdecimal::BigDecimal>, std::ops::Bound<bigdecimal::BigDecimal>);
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Range<diesel::sql_types::Date> {
    type Type = (std::ops::Bound<chrono::NaiveDate>, std::ops::Bound<chrono::NaiveDate>);
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Range<diesel::sql_types::Timestamp> {
    type Type = (std::ops::Bound<chrono::NaiveDateTime>, std::ops::Bound<chrono::NaiveDateTime>);
}

#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Range<diesel::sql_types::Timestamptz> {
    type Type = (
        std::ops::Bound<chrono::DateTime<chrono::Utc>>,
        std::ops::Bound<chrono::DateTime<chrono::Utc>>,
    );
}

// Record type (PostgreSQL)
#[cfg(feature = "postgres")]
impl RustSqlType for diesel::sql_types::Record<()> {
    type Type = ();
}
