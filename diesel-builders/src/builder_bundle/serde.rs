//! Submodule implementing serde-related traits for table bundles.
#![cfg(feature = "serde")]

use crate::{BundlableTable, HorizontalSameAsKeys, TableAddition, TableBuilderBundle};

impl<T: BundlableTable> serde::Serialize for TableBuilderBundle<T>
where
    T::InsertableModel: serde::Serialize,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Serialize,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        #[derive(serde::Serialize)]
        struct TableBuilderBundleHelper<A, B, C>
        {
            insertable_model: A,
            mandatory_associated_builders: B,
            discretionary_associated_builders: C,
        }
        let helper = TableBuilderBundleHelper {
            insertable_model: &self.insertable_model,
            mandatory_associated_builders: &self.mandatory_associated_builders,
            discretionary_associated_builders: &self.discretionary_associated_builders,
        };
        helper.serialize(serializer)
    }
}

impl<'de, T: BundlableTable> serde::Deserialize<'de> for TableBuilderBundle<T>
where
    <T as TableAddition>::InsertableModel: serde::Deserialize<'de>,
    <<T::MandatoryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Deserialize<'de>,
    <<T::DiscretionaryTriangularSameAsColumns as HorizontalSameAsKeys<T>>::ReferencedTables as crate::BuildableTables>::OptionalBuilders: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TableBuilderBundleHelper<A, B, C>
        {
            insertable_model: A,
            mandatory_associated_builders: B,
            discretionary_associated_builders: C,
        }

        let helper   = TableBuilderBundleHelper::deserialize(deserializer)?;
        Ok(TableBuilderBundle {
            insertable_model: helper.insertable_model,
            mandatory_associated_builders: helper.mandatory_associated_builders,
            discretionary_associated_builders: helper.discretionary_associated_builders,
        })
    }
}
