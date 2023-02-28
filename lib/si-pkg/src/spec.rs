use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub name: String,
    pub version: String,

    pub description: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,

    pub schemas: Vec<Schema>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub name: String,
    pub category: String,

    pub variants: Vec<SchemaVariant>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariant {
    pub name: String,
    pub link: Option<Url>,
    pub color: Option<String>,

    pub domain: Prop,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Prop {
    #[serde(rename_all = "camelCase")]
    String { name: String },
    #[serde(rename_all = "camelCase")]
    Number { name: String },
    #[serde(rename_all = "camelCase")]
    Boolean { name: String },
    #[serde(rename_all = "camelCase")]
    Map { name: String, type_prop: Box<Prop> },
    #[serde(rename_all = "camelCase")]
    Array { name: String, type_prop: Box<Prop> },
    #[serde(rename_all = "camelCase")]
    Object { name: String, entries: Vec<Prop> },
}
