use bytes::BytesMut;
use postgres_types::ToSql;

use crate::create_xxhash_type;

create_xxhash_type!(ContentHash);

impl ToSql for ContentHash {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut BytesMut,
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
        out: &mut BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let self_string = self.to_string();
        self_string.to_sql_checked(ty, out)
    }
}
