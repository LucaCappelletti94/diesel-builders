use diesel_builders::prelude::*;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ProductError {
    #[error("Stock cannot be negative")]
    NegativeStock,
}

impl From<std::convert::Infallible> for ProductError {
    fn from(inf: std::convert::Infallible) -> Self {
        match inf {}
    }
}

// ValidateColumn implementation WITHOUT #[const_validator]
// This means validate_stock() function won't be generated
impl diesel_builders::ValidateColumn<products::stock>
    for <products::table as diesel_builders::TableExt>::NewValues
{
    type Error = ProductError;

    fn validate_column(value: &i32) -> Result<(), Self::Error> {
        if *value < 0 {
            return Err(ProductError::NegativeStock);
        }
        Ok(())
    }
}

#[derive(Debug, diesel::Queryable, Clone, diesel::Selectable, diesel::Identifiable, PartialEq, TableModel)]
#[diesel(table_name = products)]
#[table_model(error = ProductError, surrogate_key)]
pub struct Product {
    id: i32,
    /// Stock with default value - but missing const_validator!
    #[table_model(default = 10)]
    stock: i32,
}

fn main() {}
