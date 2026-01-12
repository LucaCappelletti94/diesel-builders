use diesel_builders::prelude::*;

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        user_id -> Integer,
        title -> Text,
    }
}

// 2 columns vs 1 column
fk!((posts::user_id, posts::title) -> (users::id));

fn main() {}
