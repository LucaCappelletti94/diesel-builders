use diesel_builders::prelude::*;

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct UserColumnName {
    pub id: i32,
    #[diesel(column_name = real_name)]
    pub name: String,
}

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct UserEmbed {
    pub id: i32,
    #[diesel(embed)]
    pub name: String,
}

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct UserSerializeAs {
    pub id: i32,
    #[diesel(serialize_as = "SomeType")]
    pub name: String,
}

#[derive(TableModel)]
#[diesel(table_name = users)]
pub struct UserMultipleDefaults {
    pub id: i32,
    #[table_model(default = "A")]
    #[table_model(default = "B")]
    pub name: String,
}

fn main() {}
