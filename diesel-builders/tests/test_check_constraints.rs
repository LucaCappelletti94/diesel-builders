//! Test custom validation with TrySetColumn implementations
//!
//! This test demonstrates that CHECK constraints and custom TrySetColumn
//! implementations work together to validate data at both the Rust and SQL levels.

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
    fn try_set_column(&mut self, value: &String) -> anyhow::Result<&mut Self> {
        if value.trim().is_empty() {
            anyhow::bail!("Product name cannot be empty");
        }
        if value.len() > 200 {
            anyhow::bail!("Product name cannot exceed 200 characters");
        }
        self.name = Some(value.clone());
        Ok(self)
    }
}

// Validation for price - must be positive
impl diesel_builders::TrySetColumn<products::price> for NewProduct {
    fn try_set_column(&mut self, value: &i32) -> anyhow::Result<&mut Self> {
        if *value <= 0 {
            anyhow::bail!("Price must be positive");
        }
        self.price = Some(*value);
        Ok(self)
    }
}

// Validation for stock_quantity - must be non-negative
impl diesel_builders::TrySetColumn<products::stock_quantity> for NewProduct {
    fn try_set_column(&mut self, value: &i32) -> anyhow::Result<&mut Self> {
        if *value < 0 {
            anyhow::bail!("Stock quantity cannot be negative");
        }
        self.stock_quantity = Some(*value);
        Ok(self)
    }
}

impl TableAddition for products::table {
    type InsertableModel = NewProduct;
    type Model = Product;
    type InsertableColumns = (products::name, products::price, products::stock_quantity);
}

#[test]
fn test_valid_product_insert() {
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
        .try_set_column::<products::name>(&"Laptop".to_string())
        .unwrap()
        .try_set_column::<products::price>(&999)
        .unwrap()
        .try_set_column::<products::stock_quantity>(&50)
        .unwrap()
        .insert(&mut conn)
        .unwrap();

    assert_eq!(product.name, "Laptop");
    assert_eq!(product.price, 999);
    assert_eq!(product.stock_quantity, 50);
}

#[test]
fn test_empty_name_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::name>(&"   ".to_string());

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("cannot be empty"));
}

#[test]
fn test_name_too_long_rejected() {
    let long_name = "x".repeat(201);
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::name>(&long_name);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("cannot exceed 200 characters"));
}

#[test]
fn test_negative_price_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::price>(&-100);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be positive"));
}

#[test]
fn test_zero_price_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::price>(&0);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be positive"));
}

#[test]
fn test_negative_stock_rejected() {
    let builder = products::table::builder();
    let result = builder.try_set_column::<products::stock_quantity>(&-5);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("cannot be negative"));
}

#[test]
fn test_zero_stock_allowed() {
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
        .try_set_column::<products::name>(&"Out of Stock Item".to_string())
        .unwrap()
        .try_set_column::<products::price>(&1)
        .unwrap()
        .try_set_column::<products::stock_quantity>(&0)
        .unwrap()
        .insert(&mut conn)
        .unwrap();

    assert_eq!(product.stock_quantity, 0);
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
fn test_builder_chain_with_validation() {
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

    // Test that we can chain multiple try_set_column calls
    let product = products::table::builder()
        .try_set_column::<products::name>(&"Keyboard".to_string())
        .and_then(|b| b.try_set_column::<products::price>(&50))
        .and_then(|b| b.try_set_column::<products::stock_quantity>(&100))
        .and_then(|b| b.insert(&mut conn))
        .unwrap();

    assert_eq!(product.name, "Keyboard");
    assert_eq!(product.price, 50);
    assert_eq!(product.stock_quantity, 100);
}

#[test]
fn test_validation_fails_early_in_chain() -> Result<(), Box<dyn std::error::Error>> {
    // Test that if validation fails, the error is returned immediately
    let mut builder = products::table::builder();

    // Set valid name
    builder.try_set_column_ref::<products::name>(&"Mouse".to_string())?;

    // Try to set invalid price - should fail
    let result = builder.try_set_column_ref::<products::price>(&-50);
    assert!(result.is_err());

    // The error message should indicate price validation failure
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("must be positive"));
    Ok(())
}
