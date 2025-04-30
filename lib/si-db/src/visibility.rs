use serde::{
    Deserialize,
    Serialize,
};
use serde_aux::field_attributes::deserialize_number_from_string;
use si_id::ChangeSetId;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Visibility {
    // FIXME(nick): if we change the serialization name, you will have downstream problems. As we
    // get rid of standard model, we should remove "visibility_change_set_pk" entirely and just
    // use "change_set_id".
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

    // FIXME(nick): do not trust this method. It is purely used during bootstrapping and should be
    // removed.
    pub fn new_head_fake() -> Self {
        Self {
            change_set_id: Ulid::new().into(),
        }
    }
}

// NOTE(fnichol): the fact that this is now trivial is another sure sign that perhaps `Visibility`
// can be removed unless there's another strongly associated piece of data that belongs in
// `Visibility`.
impl From<ChangeSetId> for Visibility {
    fn from(value: ChangeSetId) -> Self {
        Self {
            change_set_id: value,
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
