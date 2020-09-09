use serde::{Deserialize, Serialize};
use serde_json;
use si_data::DataQuery;

#[derive(Deserialize, Serialize, Debug)]
pub enum OrderByDirection {
    ASC,
    DSC,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub query: Option<DataQuery>,
    pub page_size: Option<u32>,
    pub order_by: Option<String>,
    pub order_by_direction: Option<OrderByDirection>,
    pub page_token: Option<String>,
    pub scope_by_tenant_id: Option<String>,
    pub type_name: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct ListResponse {
    pub items: Vec<serde_json::Value>,
    pub total_count: u32,
    pub next_item_id: String,
    pub page_token: String,
}

#[derive(Deserialize, Debug)]
pub struct SetField {
    pub pointer: String,
    pub value: String,
}
