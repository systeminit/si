#[macro_export]
macro_rules! pk {
    (
        $(#[$($attrss:tt)*])*
        $name:ident
    ) => {
        $(#[$($attrss)*])*
        #[derive(
            Eq,
            PartialEq,
            PartialOrd,
            Ord,
            Copy,
            Clone,
            Hash,
            derive_more::From,
            derive_more::Into,
            derive_more::Display,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(ulid::Ulid);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0.to_string()).finish()
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::NONE
            }
        }

        impl $name {
            /// An unset id value.
            pub const NONE: Self = Self(::ulid::Ulid::nil());

            /// Returns `true` if id is set (i.e. not [`NONE`](Self::NONE)).
            pub fn is_some(&self) -> bool {
                !self.is_none()
            }

            /// Returns `true` if is unset (i.e. value is equal to [`NONE`](Self::NONE)).
            pub fn is_none(&self) -> bool {
                self == &Self::NONE
            }

            /// Generates a new key which is virtually guarenteed to be unique.
            pub fn generate() -> Self {
                Self(ulid::Ulid::new())
            }

            /// Converts type into inner Ulid
            pub fn into_inner(self) -> ::ulid::Ulid {
                self.0
            }
        }

        impl From<::si_events::ulid::Ulid> for $name {
            fn from(events_ulid: ::si_events::ulid::Ulid) -> Self {
                events_ulid.inner().into()
            }
        }

        impl From<Option<$name>> for $name {
            fn from(optional_pk: Option<$name>) -> Self {
                match optional_pk {
                    Some(id) => id,
                    None => Self::NONE,
                }
            }
        }

        impl From<$name> for String {
            fn from(pk: $name) -> Self {
                ulid::Ulid::from(pk.0).into()
            }
        }

        impl<'a> From<&'a $name> for ::ulid::Ulid {
            fn from(pk: &'a $name) -> Self {
                pk.0
            }
        }

        impl From<$name> for ::si_events::ulid::Ulid {
            fn from(pk: $name) -> Self {
                pk.0.into()
            }
        }

        impl<'a> From<&'a $name> for ::si_events::ulid::Ulid {
            fn from(pk: &'a $name) -> Self {
                pk.0.into()
            }
        }

        impl<'a> From<&'a $name> for $name {
            fn from(pk: &'a $name) -> Self {
                *pk
            }
        }

        impl std::str::FromStr for $name {
            type Err = ulid::DecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(ulid::Ulid::from_string(s)?))
            }
        }

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let id: String = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok(Self(ulid::Ulid::from_string(&id)?))
            }

            fn accepts(ty: &postgres_types::Type) -> bool {
                ty == &postgres_types::Type::BPCHAR
                    || ty.kind() == &postgres_types::Kind::Domain(postgres_types::Type::BPCHAR)
            }
        }

        impl postgres_types::ToSql for $name {
            fn to_sql(
                &self,
                ty: &postgres_types::Type,
                out: &mut postgres_types::private::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
            where
                Self: Sized,
            {
                postgres_types::ToSql::to_sql(&self.0.to_string(), ty, out)
            }

            fn accepts(ty: &postgres_types::Type) -> bool
            where
                Self: Sized,
            {
                ty == &postgres_types::Type::BPCHAR
                    || ty.kind() == &postgres_types::Kind::Domain(postgres_types::Type::BPCHAR)
            }

            fn to_sql_checked(
                &self,
                ty: &postgres_types::Type,
                out: &mut postgres_types::private::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
                postgres_types::ToSql::to_sql(&self.0.to_string(), ty, out)
            }
        }
    };
}
