//! Common utilities and shared table definitions for tests.

use diesel_builders::{Typed, prelude::*};

diesel::table! {
    /// Dogs table - extends animals via foreign key.
    dogs (id) {
        /// Primary key of the dog, foreign key to animals.id.
        id -> Integer,
        /// The breed of the dog.
        breed -> Text,
    }
}

impl Typed for dogs::id {
    type Type = i32;
}

impl Typed for dogs::breed {
    type Type = String;
}

impl diesel_builders::HorizontalSameAsGroup for dogs::id {
    type Idx = diesel_builders::typenum::U0;
    type MandatoryHorizontalKeys = ();
    type DiscretionaryHorizontalKeys = ();
}

impl diesel_builders::HorizontalSameAsGroup for dogs::breed {
    type Idx = diesel_builders::typenum::U0;
    type MandatoryHorizontalKeys = ();
    type DiscretionaryHorizontalKeys = ();
}

impl BundlableTable for dogs::table {
    type MandatoryTriangularColumns = ();
    type DiscretionaryTriangularColumns = ();
}
