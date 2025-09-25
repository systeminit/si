use dal::SecretId;
use serde::{
    Deserialize,
    Serialize,
};
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretViewV1 {
    #[schema(example = json!({
        "secretDefinition": "aws_credentials",
        "formData": [
            {
                "name": "access_key_id",
                "kind": "string"
            },
            {
                "name": "secret_access_key",
                "kind": "password"
            },
            {
                "name": "region",
                "kind": "string"
            },
            {
                "name": "default_output",
                "kind": "string"
            }
        ]
    }))]
    pub definition: SecretDefinitionV1,
    #[schema(example = json!([
        {
            "id": "01HAXYZF3GC9CYA6ZVSM3E4YHH",
            "name": "Production AWS Key",
            "definition": "aws_credentials",
            "description": "AWS credentials for production environment"
        },
        {
            "id": "01HAXYZF3GC9CYA6ZVSM3E4YHI",
            "name": "Development AWS Key",
            "definition": "aws_credentials",
            "description": "AWS credentials for development environment"
        }
    ]))]
    pub secrets: Vec<SecretV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretV1 {
    #[schema(value_type = String, example = "01HAXYZF3GC9CYA6ZVSM3E4YHH")]
    pub id: SecretId,
    #[schema(value_type = String, example = "Production AWS Key")]
    pub name: String,
    #[schema(value_type = String, example = "aws_credentials")]
    pub definition: String,
    #[schema(value_type = Option<String>, example = "AWS credentials for production environment")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretDefinitionV1 {
    #[schema(value_type = String, example = "aws_credentials")]
    pub secret_definition: String,
    #[schema(example = json!([
        {
            "name": "access_key_id",
            "kind": "string"
        },
        {
            "name": "secret_access_key",
            "kind": "password"
        }
    ]))]
    pub form_data: Vec<SecretFormDataV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SecretFormDataV1 {
    #[schema(value_type = String, example = "access_key_id")]
    pub name: String,
    #[schema(value_type = String, example = "string")]
    pub kind: String,
}
