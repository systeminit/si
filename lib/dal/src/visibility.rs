use serde::{Deserialize, Serialize};
use serde_aux::field_attributes::deserialize_number_from_string;
use ulid::Ulid;

use crate::ChangeSetId;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Visibility {
    // FIXME(nick): if we change the serialization name, we will blow the fuck up. We need to change
    // that.
    #[serde(
        rename = "visibility_change_set_pk",
        deserialize_with = "deserialize_number_from_string"
    )]
    pub change_set_id: ChangeSetId,
}

impl Visibility {
    pub fn new(change_set_id: ChangeSetId) -> Self {
        Visibility { change_set_id }
    }

    // FIXME(nick): this is bullshit.
    pub fn new_head() -> Self {
        Self {
            change_set_id: Ulid::new().into(),
        }
    }
}

impl postgres_types::ToSql for Visibility {
    fn to_sql(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
    where
        Self: Sized,
    {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }

    fn accepts(ty: &postgres_types::Type) -> bool
    where
        Self: Sized,
    {
        ty == &postgres_types::Type::JSONB
    }

    fn to_sql_checked(
        &self,
        ty: &postgres_types::Type,
        out: &mut postgres_types::private::BytesMut,
    ) -> Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let json = serde_json::to_value(self)?;
        postgres_types::ToSql::to_sql(&json, ty, out)
    }
}
