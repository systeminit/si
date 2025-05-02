use dal::SecretId;
use serde::{
    Deserialize,
    Serialize,
};
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretViewV1 {
    pub definition: SecretDefinitionV1,
    pub secrets: Vec<SecretV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretV1 {
    #[schema(value_type = String)]
    pub id: SecretId,
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = String)]
    pub definition: String,
    #[schema(value_type = String)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretDefinitionV1 {
    #[schema(value_type = String)]
    pub secret_definition: String,
    #[schema(value_type = Vec<Object>)]
    pub form_data: Vec<SecretFormDataV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretFormDataV1 {
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = String)]
    pub kind: String,
}
