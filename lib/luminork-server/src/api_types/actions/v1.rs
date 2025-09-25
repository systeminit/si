use dal::{
    ActionPrototypeId,
    ChangeSetId,
    ComponentId,
    action::prototype::ActionKind,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::ActionState;
use si_id::{
    ActionId,
    FuncRunId,
};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ActionViewV1 {
    #[schema(value_type = String)]
    pub id: ActionId,
    #[schema(value_type = String)]
    pub prototype_id: ActionPrototypeId,
    #[schema(value_type = Option<String>)]
    pub component_id: Option<ComponentId>,
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = Option<String>)]
    pub description: Option<String>,
    #[schema(value_type = String)]
    pub kind: ActionKind,
    #[schema(value_type = String)]
    pub state: ActionState,
    #[schema(value_type = String)]
    pub originating_change_set_id: ChangeSetId,
    #[schema(value_type = Option<String>)]
    pub func_run_id: Option<FuncRunId>,
}
