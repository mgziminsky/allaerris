use serde::{de, Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version<const V: u8>;
impl<const V: u8> Serialize for Version<V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(V)
    }
}
impl<'de, const V: u8> Deserialize<'de> for Version<V> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        if value == V {
            Ok(Self)
        } else {
            Err(de::Error::custom(format!("Unsupported version for modpack manifest: {value}")))
        }
    }
}
