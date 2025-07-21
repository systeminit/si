pub const XXH3_HASH_SIZE: usize = 16;

#[macro_export]
macro_rules! create_xxhash_type {
    (
        $(#[$($attrss:tt)*])*
        $name:ident
    ) => {
        $(#[$($attrss)*])*
        #[derive(Clone, Copy, Eq, PartialEq, std::hash::Hash)]
        pub struct $name([u8; $crate::xxhash_type::XXH3_HASH_SIZE]);

        impl $name {
            pub fn new(input: &[u8]) -> Self {
                Self(::xxhash_rust::xxh3::xxh3_128(input).to_le_bytes())
            }

            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }

            pub fn nil() -> Self {
                Self::new(&[0])
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let mut other_bytes = [0u8; $crate::xxhash_type::XXH3_HASH_SIZE];
                other_bytes.copy_from_slice(other.as_bytes());
                let mut self_bytes = [0u8; $crate::xxhash_type::XXH3_HASH_SIZE];
                self_bytes.copy_from_slice(self.as_bytes());

                self_bytes.cmp(&other_bytes)
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                &self.0
            }
        }

        impl From<&::serde_json::Value> for $name {
            fn from(value: &::serde_json::Value) -> Self {
                let input = value.to_string();
                Self::new(input.as_bytes())
            }
        }

        impl From<&str> for $name {
            fn from(input: &str) -> Self {
                Self::new(input.as_bytes())
            }
        }

        impl From<u128> for $name {
            fn from(value: u128) -> Self {
                Self(value.to_le_bytes())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new("".as_bytes())
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, std::concat!(stringify!($name), "({})"), &self)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                let self_u128 = u128::from_le_bytes(self.0);
                write!(f, "{:032x}", self_u128)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_bytes(&self.0)
            }
        }

        paste::paste! {
            struct [<$name Visitor>];

            impl<'de> ::serde::de::Visitor<'de> for [<$name Visitor>] {
                type Value = $name;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("a 16 byte slice representing an xxh3 hash")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    if v.len() != $crate::xxhash_type::XXH3_HASH_SIZE {
                        return Err(E::custom(std::concat!("deserializer received wrong sized byte slice when attempting to deserialize a ", stringify!($name))));
                    }

                    let mut hash_bytes = [0u8; $crate::xxhash_type::XXH3_HASH_SIZE];
                    hash_bytes.copy_from_slice(v);

                    Ok($name(hash_bytes))
                }

                fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
                where
                    V: ::serde::de::SeqAccess<'de>,
                {
                    use ::serde::de::Error;

                    if seq.size_hint().is_some() && seq.size_hint() != Some($crate::xxhash_type::XXH3_HASH_SIZE) {
                        return Err(V::Error::custom(std::concat!("deserializer received wrong sized byte slice when attempting to deserialize a ", stringify!($name))));
                    }
                    let mut hash_bytes = Vec::with_capacity($crate::xxhash_type::XXH3_HASH_SIZE);
                    while let Some(v) = seq.next_element()? {
                        hash_bytes.push(v);
                    }
                    if hash_bytes.len() != $crate::xxhash_type::XXH3_HASH_SIZE {
                        return Err(V::Error::custom(std::concat!("deserializer received wrong sized byte slice when attempting to deserialize a ", stringify!($name))));
                    }

                    let mut bytes_slice = [0u8; $crate::xxhash_type::XXH3_HASH_SIZE];
                    bytes_slice.copy_from_slice(&hash_bytes);

                    Ok($name(bytes_slice))
                }
            }

            impl<'de> ::serde::Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    deserializer.deserialize_bytes([<$name Visitor>])
                }
            }

            #[derive(Debug, ::thiserror::Error)]
            #[error("failed to parse hash hex string")]
            pub struct [<$name ParseError>](#[from] ::std::num::ParseIntError);

            impl ::std::str::FromStr for $name {
                type Err = [<$name ParseError>];

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let hash_u128 = u128::from_str_radix(s, 16)?;
                    Ok(hash_u128.into())
                }
            }

            impl $name {
                /// Provide a [`hasher`]([<$name Hasher>]) to create [`hashes`]([<$name Hasher>]).
                pub fn hasher() -> [<$name Hasher>] {
                    [<$name Hasher>]::new()
                }
            }

            #[derive(Default)]
            pub struct [<$name Hasher>](Box<::xxhash_rust::xxh3::Xxh3>);

            impl ::std::fmt::Debug for [<$name Hasher>] {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, stringify!([<$name Hasher>]))
                }
            }

            impl [<$name Hasher>] {
                pub fn new() -> Self {
                    Self(Box::default())
                }

                pub fn update(&mut self, input: &[u8]) {
                    self.0.update(input);
                }

                pub fn finalize(&self) -> $name {
                    $name(self.0.digest128().to_le_bytes())
                }
            }
        }
    }
}
