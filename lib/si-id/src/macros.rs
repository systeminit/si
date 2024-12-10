/// The primary macro used for creating IDs for SI.
macro_rules! id {
    (
        $(#[$($attrs:tt)*])*
        $name:ident
    ) => {
        do_not_use_directly_id_inner!(
            $(#[$($attrs)*])*
            $name
        );
    };
}

/// Provides PostgreSQL type conversion implementations in addition to standard [`id!`] functionality.
macro_rules! id_with_pg_types {
    (
        $(#[$($attrs:tt)*])*
        $name:ident
    ) => {
        do_not_use_directly_id_inner!(
            $(#[$($attrs)*])*
            $name
        );

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let id: String = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok(Self(::ulid::Ulid::from_string(&id)?))
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

/// **Deprecated:** this macro provides additional functionality for using the "nil" ID on top of the standard `id!`
/// macro.
///
/// This absolutely should not be used for any new IDs. Nick will be mad. No, I'm not writing this in third
/// person, why do you ask?
macro_rules! id_with_none {
    (
        $(#[$($attrs:tt)*])*
        $name:ident
    ) => {
        do_not_use_directly_id_inner!(
            $(#[$($attrs)*])*
            $name
        );

        impl Default for $name {
            fn default() -> Self {
                Self::NONE
            }
        }

        impl $name {
            /// The "nil" value for this ID.
            pub const NONE: Self = Self(::ulid::Ulid::nil());
        }
    };
}

/// **Deprecated:** this macro provides additional functionality for using the "nil" ID on top of the standard `id!`
/// macro. It also adds support for PostgreSQL type conversions.
///
/// This absolutely should not be used for any new IDs. Nick will be mad. No, I'm not writing this in third person, why
/// do you ask?
macro_rules! id_with_none_and_pg_types {
    (
        $(#[$($attrs:tt)*])*
        $name:ident
    ) => {
        do_not_use_directly_id_inner!(
            $(#[$($attrs)*])*
            $name
        );

        impl Default for $name {
            fn default() -> Self {
                Self::NONE
            }
        }

        impl $name {
            /// The "nil" value for this ID.
            pub const NONE: Self = Self(::ulid::Ulid::nil());
        }

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let id: String = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok(Self(::ulid::Ulid::from_string(&id)?))
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

// NOTE(nick,jacob): this is a bit messy, but it would be complicated/tricky to scope this properly until stable
// Rust provides the ability to use normal visibility modifiers in front of a "macro_rules!" definition. Some escape
// hatches include, but are not limited to, creating a "si-id-macros" crate with mixed usages of "macro_export" and
// "macro_use"... or using the "macro-vis" crate (https://github.com/kestrer/macro-vis). At the time of writing,
// pursuing an escape hatch or different solution is unlikely to be worth it.
#[allow(missing_docs)]
macro_rules! do_not_use_directly_id_inner {
    (
        $(#[$($attrss:tt)*])*
        $name:ident
    ) => {
        $(#[$($attrss)*])*
        #[allow(missing_docs)]
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
        pub struct $name(::ulid::Ulid);

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0.to_string()).finish()
            }
        }

        impl $name {
            /// Length of a string-encoded ID in bytes.
            pub const ID_LEN: usize = ::ulid::ULID_LEN;

            /// Generates a new key which is virtually guaranteed to be unique.
            pub fn generate() -> Self {
                Self(::ulid::Ulid::new())
            }

            /// Calls [`Self::generate`].
            #[allow(clippy::new_without_default)]
            pub fn new() -> Self {
                Self::generate()
            }

            /// Converts type into inner [`Ulid`](::ulid::Ulid).
            pub fn into_inner(self) -> ::ulid::Ulid {
                self.0
            }

            /// Creates a Crockford Base32 encoded string that represents this Ulid.
            pub fn array_to_str<'buf>(&self, buf: &'buf mut [u8; ::ulid::ULID_LEN]) -> &'buf mut str {
                self.0.array_to_str(buf)
            }

            /// Converts the ID into a byte array.
            pub fn array_to_str_buf() -> [u8; ::ulid::ULID_LEN] {
                [0; ::ulid::ULID_LEN]
            }

            /// Constructs a new instance of Self from the given raw identifier.
            ///
            /// This function is typically used to consume ownership of the specified identifier.
            pub fn from_raw_id(value: ::ulid::Ulid) -> Self {
                Self(value)
            }

            /// Extracts the raw identifier.
            ///
            /// This function is typically used to borrow an owned idenfier.
            pub fn as_raw_id(&self) -> ::ulid::Ulid {
                self.0
            }

            /// Consumes this object, returning the raw underlying identifier.
            ///
            /// This function is typically used to transfer ownership of the underlying identifier
            /// to the caller.
            pub fn into_raw_id(self) -> ::ulid::Ulid {
                self.0
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                ::ulid::Ulid::from(id.0).into()
            }
        }

        impl<'a> From<&'a $name> for ::ulid::Ulid {
            fn from(id: &'a $name) -> Self {
                id.0
            }
        }

        impl From<$name> for crate::ulid::Ulid {
            fn from(id: $name) -> Self {
                id.0.into()
            }
        }

        impl<'a> From<&'a $name> for crate::ulid::Ulid {
            fn from(id: &'a $name) -> Self {
                id.0.into()
            }
        }

        impl From<crate::ulid::Ulid> for $name {
            fn from(ulid: crate::ulid::Ulid) -> Self {
                ulid.inner().into()
            }
        }

        impl<'a> From<&'a $name> for $name {
            fn from(id: &'a $name) -> Self {
                *id
            }
        }

        impl std::str::FromStr for $name {
            type Err = ::ulid::DecodeError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok(Self(::ulid::Ulid::from_string(s)?))
            }
        }
    };
}
