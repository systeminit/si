use dal::{
    ChangeSetId,
    ChangeSetStatus,
};
use serde::{
    Deserialize,
    Serialize,
};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChangeSetViewV1 {
    #[schema(value_type = String)]
    pub id: ChangeSetId,
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = String)]
    pub status: ChangeSetStatus,
    #[schema(value_type = bool)]
    pub is_head: bool,
}
