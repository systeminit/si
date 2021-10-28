#[macro_export]
macro_rules! pk {
    ($name:ident) => {
        #[derive(
            Debug,
            Eq,
            PartialEq,
            Copy,
            Clone,
            derive_more::From,
            derive_more::Into,
            derive_more::Display,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub struct $name(i64);

        impl<'a> postgres_types::FromSql<'a> for $name {
            fn from_sql(
                ty: &postgres_types::Type,
                raw: &'a [u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let number: i64 = postgres_types::FromSql::from_sql(ty, raw)?;
                Ok($name(number))
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
    };
}
