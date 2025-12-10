//! Submodule implementing serde-related traits for table bundles.
#![cfg(feature = "serde")]

use crate::{TableBuilderBundle, TableExt, builder_bundle::BundlableTableExt};

impl<T: BundlableTableExt + TableExt> serde::Serialize for TableBuilderBundle<T>
where
    T::InsertableModel: serde::Serialize,
    T::OptionalMandatoryNestedBuilders: serde::Serialize,
    T::OptionalDiscretionaryNestedBuilders: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct TableBuilderBundleHelper<A, B, C> {
            insertable_model: A,
            nested_mandatory_associated_builders: B,
            nested_discretionary_associated_builders: C,
        }
        let helper = TableBuilderBundleHelper {
            insertable_model: &self.insertable_model,
            nested_mandatory_associated_builders: &self.nested_mandatory_associated_builders,
            nested_discretionary_associated_builders: &self
                .nested_discretionary_associated_builders,
        };
        helper.serialize(serializer)
    }
}

impl<'de, T: BundlableTableExt + TableExt> serde::Deserialize<'de> for TableBuilderBundle<T>
where
    T::InsertableModel: serde::Deserialize<'de>,
    T::OptionalMandatoryNestedBuilders: serde::Deserialize<'de>,
    T::OptionalDiscretionaryNestedBuilders: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TableBuilderBundleHelper<A, B, C> {
            insertable_model: A,
            nested_mandatory_associated_builders: B,
            nested_discretionary_associated_builders: C,
        }

        let helper = TableBuilderBundleHelper::deserialize(deserializer)?;
        Ok(TableBuilderBundle {
            insertable_model: helper.insertable_model,
            nested_mandatory_associated_builders: helper.nested_mandatory_associated_builders,
            nested_discretionary_associated_builders: helper
                .nested_discretionary_associated_builders,
        })
    }
}
