#![cfg(feature = "serde")]
//! Submodule providing serde implementations for table builders.

use crate::{BuildableTable, BundlableTables, TableBuilder};

impl<T: BuildableTable> serde::Serialize for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: serde::Serialize,
{
    #[inline]
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as serde::ser::Serializer>::Ok, <S as serde::ser::Serializer>::Error>
    where
        S: serde::ser::Serializer,
    {
        self.bundles.serialize(serializer)
    }
}

impl<'de, T: BuildableTable> serde::Deserialize<'de> for TableBuilder<T>
where
    <T::AncestorsWithSelf as BundlableTables>::BuilderBundles: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let bundles =
            <T::AncestorsWithSelf as BundlableTables>::BuilderBundles::deserialize(deserializer)?;
        Ok(Self { bundles })
    }
}
