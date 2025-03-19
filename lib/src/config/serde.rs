use serde::{Deserializer, Serializer, de::Visitor, ser::SerializeMap};

/// SerDe a list of profiles as a map of `path` => `name` pairs
pub(super) mod profiles {
    use super::*;
    use crate::config::{Profile, ProfilesList};

    pub fn serialize<S, Iter, Item>(data: Iter, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        Item: AsRef<Profile>,
        Iter: IntoIterator<Item = Item>,
        Iter::IntoIter: ExactSizeIterator,
    {
        let data = data.into_iter();
        let mut map = ser.serialize_map(Some(data.len()))?;
        for p in data {
            let Profile { name, path, .. } = p.as_ref();
            map.serialize_entry(path, name)?;
        }
        map.end()
    }

    type Value = ProfilesList;

    pub fn deserialize<'de, D>(de: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ProfilesVisitor;
        impl<'de> Visitor<'de> for ProfilesVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a single valued 'map' of 'path' => 'name'")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut out = Value::new();
                loop {
                    match map.next_entry() {
                        Ok(Some((path, name))) => {
                            out.insert(Profile::new(name, path).into());
                        },
                        Err(_) => { /* Invalid Profile - Skip */ },
                        Ok(None) => break,
                    }
                }
                Ok(out)
            }
        }

        de.deserialize_map(ProfilesVisitor)
    }
}
