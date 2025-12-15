use diesel_builders::prelude::*;


#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = discretionary_table)]
#[table_model(error = std::convert::Infallible)]
pub struct Discretionary {
	#[infallible]
	id: i32,
	#[infallible]
	parent_id: i32,
	#[infallible]
	discretionary_field: String,
}

#[derive(Debug, Queryable, Clone, Selectable, Identifiable, PartialEq, TableModel)]
#[diesel(table_name = child_table)]
#[table_model(error = std::convert::Infallible)]
pub struct Child {
	// Resolve ambiguity by specifying the key column (disc_id_1)
	#[same_as(discretionary_table::parent_id, disc_id_1)]
	id: i32,
	
	#[discretionary(discretionary_table)]
	disc_id_1: i32,
	
	// Use the second key for another field to make it a valid HorizontalKey
	#[same_as(discretionary_table::discretionary_field, disc_id_2)]
	other_field: String,

	#[discretionary(discretionary_table)]
	disc_id_2: i32,
}

fn main() {
    // This should compile successfully
}
