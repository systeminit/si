#[macro_export]
macro_rules! content_address {
    (
        $(#[$($attrss:tt)*])*
        $name:ident
    ) => {
        $(#[$($attrss)*])*
        #[derive(Clone, Copy, Eq, PartialEq, std::hash::Hash)]
        pub struct $name(::blake3::Hash);

        impl $name {
            pub fn new(input: &[u8]) -> Self {
                Self(blake3::hash(input))
            }

            pub fn as_bytes(&self) -> &[u8] {
                self.0.as_bytes()
            }

            pub fn nil() -> Self {
                Self(blake3::Hash::from_bytes([0; 32]))
            }
        }

        impl PartialOrd for $name {
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $name {
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                let mut other_bytes = [0u8; 32];
                other_bytes.copy_from_slice(other.as_bytes());
                let mut self_bytes = [0u8; 32];
                self_bytes.copy_from_slice(self.as_bytes());

                self_bytes.cmp(&other_bytes)
            }
        }

        impl AsRef<[u8]> for $name {
            fn as_ref(&self) -> &[u8] {
                self.0.as_bytes()
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

        impl Default for $name {
            fn default() -> Self {
                Self::new("".as_bytes())
            }
        }

        impl ::std::fmt::Debug for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, std::concat!(stringify!($name), "({})"), self.0)
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(&self.to_string())
            }
        }

        paste::paste! {
            struct [<$name Visitor>];

            impl<'de> ::serde::de::Visitor<'de> for [<$name Visitor>] {
                type Value = $name;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("a blake3 hash string")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    use ::std::str::FromStr;

                    $name::from_str(v).map_err(|e| E::custom(e.to_string()))
                }
            }

            impl<'de> ::serde::Deserialize<'de> for $name {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::serde::Deserializer<'de>,
                {
                    deserializer.deserialize_str([<$name Visitor>])
                }
            }

            #[derive(Debug, ::thiserror::Error)]
            #[error("failed to parse hash hex string")]
            pub struct [<$name ParseError>](#[from] ::blake3::HexError);

            impl ::std::str::FromStr for $name {
                type Err = [<$name ParseError>];

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Ok(Self(blake3::Hash::from_str(s)?))
                }
            }

            impl $name {
                /// Provide a [`hasher`](ContentHasher) to create [`hashes`](ContentHash).
                pub fn hasher() -> [<$name Hasher>] {
                    [<$name Hasher>]::new()
                }
            }

            #[derive(Debug, Default)]
            pub struct [<$name Hasher>](::blake3::Hasher);

            impl [<$name Hasher>] {
                pub fn new() -> Self {
                    Self(::blake3::Hasher::new())
                }

                pub fn update(&mut self, input: &[u8]) {
                    self.0.update(input);
                }

                pub fn finalize(&self) -> $name {
                    $name(self.0.finalize())
                }
            }

            struct [<$name BytesVisitor>];

            impl<'de> ::serde::de::Visitor<'de> for [<$name BytesVisitor>] {
                type Value = $name;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("a blake3 hash byte slice")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    if v.len() != 32 {
                        return Err(E::custom(std::concat!("deserializer received wrong sized byte slice when attempting to deserialize a ", stringify!($name))));
                    }

                    let mut hash_bytes = [0u8; 32];
                    hash_bytes.copy_from_slice(v);

                    Ok($name(::blake3::Hash::from_bytes(hash_bytes)))
                }
            }

            pub fn [<deserialize_ $name:snake _as_bytes>]<'de, D>(d: D) -> Result<$name, D::Error>
            where D: ::serde::de::Deserializer<'de>
            {
                d.deserialize_bytes([<$name BytesVisitor>])
            }

            pub fn [<serialize_ $name:snake _as_bytes>]<S>(value: &$name, serializer: S) -> Result<S::Ok, S::Error>
            where S: ::serde::ser::Serializer,
            {
                serializer.serialize_bytes(value.as_bytes())
            }
        }

        impl ::postgres_types::ToSql for $name {
            fn to_sql(
                &self,
                ty: &postgres_types::Type,
                out: &mut ::bytes::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
            where
                Self: Sized,
            {
                let self_string = self.to_string();

                self_string.to_sql(ty, out)
            }

            fn accepts(ty: &postgres_types::Type) -> bool
            where
                Self: Sized,
            {
                String::accepts(ty)
            }

            fn to_sql_checked(
                &self,
                ty: &postgres_types::Type,
                out: &mut ::bytes::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
                let self_string = self.to_string();
                self_string.to_sql_checked(ty, out)
            }
        }

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                use ::std::str::FromStr;

                let hash_string: String = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok(Self(blake3::Hash::from_str(&hash_string)?))
            }

            fn accepts(ty: &postgres_types::Type) -> bool {
                ty == &postgres_types::Type::TEXT
                    || ty.kind() == &postgres_types::Kind::Domain(postgres_types::Type::TEXT)
            }
        }
    };
}
