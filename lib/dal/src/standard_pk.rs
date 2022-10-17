#[macro_export]
macro_rules! pk {
    ($name:ident) => {
        #[derive(
            Debug,
            Eq,
            PartialEq,
            Copy,
            Clone,
            Hash,
            derive_more::From,
            derive_more::Into,
            derive_more::Display,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(i64);

        impl $name {
            pub const NONE: Self = Self(-1);

            pub fn is_some(&self) -> bool {
                !self.is_none()
            }

            pub fn is_none(&self) -> bool {
                self == &Self::NONE
            }
        }

        impl<'a> From<&'a $name> for i64 {
            fn from(pk: &'a $name) -> Self {
                pk.0
            }
        }

        impl std::str::FromStr for $name {
            type Err = std::num::ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let x = s.parse::<i64>()?;
                Ok(Self(x))
            }
        }

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let number: i64 = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok(Self(number))
            }

            fn accepts(ty: &postgres_types::Type) -> bool {
                ty == &postgres_types::Type::INT8
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
                postgres_types::ToSql::to_sql(&self.0, &ty, out)
            }

            fn accepts(ty: &postgres_types::Type) -> bool
            where
                Self: Sized,
            {
                ty == &postgres_types::Type::INT8
            }

            fn to_sql_checked(
                &self,
                ty: &postgres_types::Type,
                out: &mut postgres_types::private::BytesMut,
            ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
                postgres_types::ToSql::to_sql(&self.0, &ty, out)
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
    };
}
