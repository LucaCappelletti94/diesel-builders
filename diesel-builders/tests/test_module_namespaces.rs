//! Independent model namespaces own their query compatibility.

macro_rules! models {
    () => {
        use diesel_builders::prelude::*;
        #[derive(Clone, Queryable, Selectable, Identifiable, TableModel)]
        #[diesel(table_name = parents)]
        #[table_model(surrogate_key)]
        pub struct Parent { id: i32, name: String }

        #[derive(Clone, Queryable, Selectable, Identifiable, TableModel)]
        #[diesel(table_name = children)]
        #[table_model(ancestors(parents))]
        pub struct Child { id: i32, note: String }

        diesel::allow_tables_to_appear_in_same_query!(parents, children);

        pub fn exercise() -> Result<(), Box<dyn std::error::Error>> {
            let mut conn = SqliteConnection::establish(":memory:")?;
            diesel::sql_query("CREATE TABLE parents(id INTEGER PRIMARY KEY, name TEXT NOT NULL)").execute(&mut conn)?;
            diesel::sql_query("CREATE TABLE children(id INTEGER PRIMARY KEY REFERENCES parents(id), note TEXT NOT NULL)").execute(&mut conn)?;
            let child = children::table::builder().name("Parent").note("Child").insert(&mut conn)?;
            let parent: Parent = child.ancestor(&mut conn)?;
            assert_eq!(parent.name(), "Parent");
            assert_eq!(child.note(), "Child");
            Ok(())
        }
    };
}
mod first {
    models!();
}
mod second {
    models!();
}

#[test]
fn test_independent_same_named_models() -> Result<(), Box<dyn std::error::Error>> {
    first::exercise()?;
    second::exercise()?;
    Ok(())
}
