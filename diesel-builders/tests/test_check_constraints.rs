//! Test custom validation with TrySetColumn implementations
//!
//! This test demonstrates that CHECK constraints and custom TrySetColumn
//! implementations work together to validate data at both the Rust and SQL levels.

mod common;
use diesel::prelude::*;
use diesel_builders::prelude::*;
use diesel_builders_macros::{GetColumn, HasTable, MayGetColumn, Root, TableModel};

diesel::table! {
    /// Products table schema
    products (id) {
        /// Product ID
        id -> Integer,
        /// Product name
        name -> Text,
        /// Product price in cents
        price -> Integer,
        /// Available stock quantity
        stock_quantity -> Integer,
    }
}

/// Product model representing a row in the products table
#[derive(Debug, Queryable, Clone, Selectable, Identifiable, GetColumn, Root, TableModel)]
#[diesel(table_name = products)]
pub struct Product {
    /// Product ID
    pub id: i32,
    /// Product name
    pub name: String,
    /// Product price in cents
    pub price: i32,
    /// Available stock quantity
    pub stock_quantity: i32,
}

/// New product builder for inserting products
#[derive(Debug, Default, Clone, Insertable, MayGetColumn, HasTable)]
#[diesel(table_name = products)]
pub struct NewProduct {
    /// Product name
    pub name: Option<String>,
    /// Product price in cents
    pub price: Option<i32>,
    /// Available stock quantity
    pub stock_quantity: Option<i32>,
}

// Validation for product name - non-empty, max 200 chars
impl diesel_builders::TrySetColumn<products::name> for NewProduct {
    type Error = NewProductError;

    fn try_set_column(&mut self, value: String) -> Result<&mut Self, Self::Error> {
        if value.trim().is_empty() {
            return Err(NewProductError::NameEmpty);
        }
        if value.len() > 200 {
            return Err(NewProductError::NameTooLong);
        }
        self.name = Some(value);
        Ok(self)
    }
}

// Single error enum for NewProduct validations
/// Error variants returned when setting fields on `NewProduct`.
#[derive(Debug, PartialEq, Eq)]
pub enum NewProductError {
    /// Name cannot be empty
    NameEmpty,
    /// Name too long
    NameTooLong,
    /// Price must be positive
    PriceNonPositive,
    /// Stock cannot be negative
    StockNegative,
}

impl std::fmt::Display for NewProductError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewProductError::NameEmpty => write!(f, "Product name cannot be empty"),
            NewProductError::NameTooLong => write!(f, "Product name cannot exceed 200 characters"),
            NewProductError::PriceNonPositive => write!(f, "Price must be positive"),
            NewProductError::StockNegative => write!(f, "Stock quantity cannot be negative"),
        }
    }
}

impl std::error::Error for NewProductError {}

impl From<std::convert::Infallible> for NewProductError {
    fn from(inf: std::convert::Infallible) -> Self {
        match inf {}
    }
}

// Validation for price - must be positive
impl diesel_builders::TrySetColumn<products::price> for NewProduct {
    type Error = NewProductError;

    fn try_set_column(&mut self, value: i32) -> Result<&mut Self, Self::Error> {
        if value <= 0 {
            return Err(NewProductError::PriceNonPositive);
        }
        self.price = Some(value);
        Ok(self)
    }
}

// Validation for stock_quantity - must be non-negative
impl diesel_builders::TrySetColumn<products::stock_quantity> for NewProduct {
    type Error = NewProductError;

    fn try_set_column(&mut self, value: i32) -> Result<&mut Self, Self::Error> {
        if value < 0 {
            return Err(NewProductError::StockNegative);
        }
        self.stock_quantity = Some(value);
        Ok(self)
    }
}

impl TableAddition for products::table {
    type InsertableModel = NewProduct;
    type Model = Product;
    type InsertableColumns = (products::name, products::price, products::stock_quantity);
}

#[test]
fn test_valid_product_insert() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    diesel::sql_query(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
            price INTEGER NOT NULL CHECK(price > 0),
            stock_quantity INTEGER NOT NULL CHECK(stock_quantity >= 0)
        )",
    )
    .execute(&mut conn)
    .unwrap();

    let product: Product = products::table::builder()
        .try_set_column::<products::name>("Laptop")?
        .try_set_column::<products::price>(999)?
        .try_set_column::<products::stock_quantity>(50)?
        .insert(&mut conn)?;

    assert_eq!(product.name, "Laptop");
    assert_eq!(product.price, 999);
    assert_eq!(product.stock_quantity, 50);
    Ok(())
}

#[test]
fn test_empty_name_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::name>("   ");

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("cannot be empty"));
}

#[test]
fn test_name_too_long_rejected() {
    let long_name = "x".repeat(201);
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::name>(long_name);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("cannot exceed 200 characters"));
}

#[test]
fn test_negative_price_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::price>(-100);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be positive"));
}

#[test]
fn test_zero_price_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::price>(0);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be positive"));
}

#[test]
fn test_negative_stock_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::stock_quantity>(-5);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("cannot be negative"));
}

#[test]
fn test_zero_stock_allowed() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    diesel::sql_query(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
            price INTEGER NOT NULL CHECK(price > 0),
            stock_quantity INTEGER NOT NULL CHECK(stock_quantity >= 0)
        )",
    )
    .execute(&mut conn)
    .unwrap();

    let product: Product = products::table::builder()
        .try_set_column::<products::name>("Out of Stock Item")?
        .try_set_column::<products::price>(1)?
        .try_set_column::<products::stock_quantity>(0)?
        .insert(&mut conn)?;

    assert_eq!(product.stock_quantity, 0);

    Ok(())
}

#[test]
fn test_check_constraint_enforced_by_database() {
    let mut conn = SqliteConnection::establish(":memory:").unwrap();

    diesel::sql_query(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
            price INTEGER NOT NULL CHECK(price > 0),
            stock_quantity INTEGER NOT NULL CHECK(stock_quantity >= 0)
        )",
    )
    .execute(&mut conn)
    .unwrap();

    // Try to insert directly via SQL with invalid data (bypassing Rust validation)
    let result = diesel::sql_query(
        "INSERT INTO products (name, price, stock_quantity) VALUES ('Test', -100, 0)",
    )
    .execute(&mut conn);

    // Database CHECK constraint should reject this
    assert!(result.is_err());
}

#[test]
fn test_builder_chain_with_validation() -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = common::establish_test_connection()?;

    diesel::sql_query(
        "CREATE TABLE products (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL CHECK(length(trim(name)) > 0 AND length(name) <= 200),
            price INTEGER NOT NULL CHECK(price > 0),
            stock_quantity INTEGER NOT NULL CHECK(stock_quantity >= 0)
        )",
    )
    .execute(&mut conn)?;

    // Test that we can chain multiple try_set_column calls
    let product = products::table::builder()
        .try_set_column::<products::name>("Keyboard")?
        .try_set_column::<products::price>(50)?
        .try_set_column::<products::stock_quantity>(100)?
        .insert(&mut conn)?;

    assert_eq!(product.name, "Keyboard");
    assert_eq!(product.price, 50);
    assert_eq!(product.stock_quantity, 100);

    Ok(())
}

#[test]
fn test_validation_fails_early_in_chain() -> Result<(), Box<dyn std::error::Error>> {
    // Test that if validation fails, the error is returned immediately
    let mut builder = products::table::builder();

    // Set valid name
    builder.try_set_column_ref::<products::name>("Mouse")?;

    // Try to set invalid price - should fail
    let result = builder.try_set_column_ref::<products::price>(-50);
    assert!(result.is_err());

    // The error message should indicate price validation failure
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be positive"));
    Ok(())
}
