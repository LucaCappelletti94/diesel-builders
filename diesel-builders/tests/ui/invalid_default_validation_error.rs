//! Test that default values are validated at compile time when using const_validator.
//!
//! This test verifies that when a failable column has a default value that doesn't
//! pass the validation defined in ValidateColumn, the compilation fails with a clear
//! error message.

use diesel_builders::prelude::*;
use std::convert::Infallible;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProductError {
    #[error("Stock cannot be negative")]
    StockNegative,
}

impl From<Infallible> for ProductError {
    fn from(inf: Infallible) -> Self {
        match inf {}
    }
}

#[diesel_builders_macros::const_validator]
impl diesel_builders::ValidateColumn<products::stock>
    for <products::table as diesel_builders::TableExt>::NewValues
{
    type Error = ProductError;

    fn validate_column(value: &i32) -> Result<(), Self::Error> {
        if *value < 0 {
            return Err(ProductError::StockNegative);
        }
        Ok(())
    }
}

#[derive(Debug, diesel::Queryable, Clone, diesel::Selectable, diesel::Identifiable, PartialEq, TableModel)]
#[diesel(table_name = products)]
#[table_model(error = ProductError, surrogate_key)]
pub struct Product {
    id: i32,
    /// Stock with invalid negative default value - should fail at compile time
    #[table_model(default = -10)]
    stock: i32,
}

fn main() {}
