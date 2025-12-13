//! Submodule implementing serde-related traits for table bundles.
#![cfg(feature = "serde")]

use crate::{TableBuilderBundle, builder_bundle::BundlableTableExt};

impl<T: BundlableTableExt> serde::Serialize for TableBuilderBundle<T>
where
    T::NewValues: serde::Serialize,
    T::OptionalMandatoryNestedBuilders: serde::Serialize,
    T::OptionalDiscretionaryNestedBuilders: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct TableBuilderBundleHelper<A, B, C> {
            /// Owned representation of the insertable model contained in the
            /// bundle, used for serialization.
            insertable_model: A,
            /// Optional nested mandatory associated builders; serialized as a
            /// structure matching the insertable model's nested builder layout.
            nested_mandatory_associated_builders: B,
            /// Optional nested discretionary associated builders; serialized as
            /// a structure matching the insertable model's nested builder
            /// layout.
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

impl<'de, T: BundlableTableExt> serde::Deserialize<'de> for TableBuilderBundle<T>
where
    T::NewValues: serde::Deserialize<'de>,
    T::OptionalMandatoryNestedBuilders: serde::Deserialize<'de>,
    T::OptionalDiscretionaryNestedBuilders: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct TableBuilderBundleHelper<A, B, C> {
            /// Owned representation of the insertable model contained in the
            /// bundle, used for deserialization.
            insertable_model: A,
            /// Optional nested mandatory associated builders; deserialized to
            /// match the insertable model's nested builder layout.
            nested_mandatory_associated_builders: B,
            /// Optional nested discretionary associated builders; deserialized
            /// to match the insertable model's nested builder layout.
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
