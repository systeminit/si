use ulid::Ulid as CoreUlid;

const ULID_SIZE: usize = 16;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, Default, PartialOrd, Ord)]
pub struct Ulid(CoreUlid);

impl Ulid {
    pub fn new() -> Self {
        Self(CoreUlid::new())
    }

    pub fn inner(&self) -> CoreUlid {
        self.0
    }
}
struct UlidVisitor;

impl<'de> ::serde::de::Visitor<'de> for UlidVisitor {
    type Value = Ulid;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("a 16 byte slice representing an xxh3 hash")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: ::serde::de::Error,
    {
        if v.len() != ULID_SIZE {
            return Err(E::custom(std::concat!(
                "deserializer received wrong sized byte slice when attempting to deserialize a ",
                stringify!($name)
            )));
        }

        let mut ulid_bytes = [0u8; ULID_SIZE];
        ulid_bytes.copy_from_slice(v);

        Ok(Ulid(CoreUlid::from_bytes(ulid_bytes)))
    }
}

impl<'de> ::serde::Deserialize<'de> for Ulid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_bytes(UlidVisitor)
    }
}
impl ::serde::Serialize for Ulid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_bytes(&self.0.to_bytes())
    }
}

impl std::fmt::Display for Ulid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ulid({})", self.0.to_string())
    }
}

impl From<CoreUlid> for Ulid {
    fn from(value: CoreUlid) -> Self {
        Ulid(value)
    }
}

impl From<Ulid> for CoreUlid {
    fn from(value: Ulid) -> Self {
        value.0
    }
}
