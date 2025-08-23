use axum::{
    Json,
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use dal::{
    FuncId,
    PropId,
    SchemaId,
    SchemaVariantId,
    TransactionsError,
    prop::PropError,
};
use frigg::FriggError;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::prop_schema::PropSchemaV1 as CachedPropSchemaV1;
use thiserror::Error;
use utoipa::ToSchema;

use crate::AppState;

pub mod find_schema;
pub mod get_default_variant;
pub mod get_schema;
pub mod get_variant;
pub mod list_schemas;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("decode error: {0}")]
    Decode(#[from] ulid::DecodeError),
    #[error("frigg error: {0}")]
    Frigg(#[from] FriggError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema not found by name: {0}")]
    SchemaNameNotFound(String),
    #[error("schema not found error: {0}")]
    SchemaNotFound(SchemaId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("schema variant not found error: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("schema variant {0} not a variant for the schema {1} error")]
    SchemaVariantNotMemberOfSchema(SchemaId, SchemaVariantId),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

pub type SchemaResult<T> = Result<T, SchemaError>;

#[derive(Deserialize, ToSchema)]
pub struct SchemaV1RequestPath {
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
}

#[derive(Deserialize, ToSchema)]
pub struct SchemaVariantV1RequestPath {
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String)]
    pub schema_variant_id: SchemaVariantId,
}

impl IntoResponse for SchemaError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl crate::service::v1::common::ErrorIntoResponse for SchemaError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            SchemaError::SchemaNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaNameNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaVariantNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaVariantNotMemberOfSchema(_, _) => {
                (StatusCode::PRECONDITION_REQUIRED, self.to_string())
            }
            SchemaError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

impl From<PropError> for SchemaError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<JsonRejection> for SchemaError {
    fn from(rejection: JsonRejection) -> Self {
        match rejection {
            JsonRejection::JsonDataError(_) => {
                SchemaError::Validation(format!("Invalid JSON data format: {rejection}"))
            }
            JsonRejection::JsonSyntaxError(_) => {
                SchemaError::Validation(format!("Invalid JSON syntax: {rejection}"))
            }
            JsonRejection::MissingJsonContentType(_) => SchemaError::Validation(
                "Request must have Content-Type: application/json header".to_string(),
            ),
            _ => SchemaError::Validation(format!("JSON validation error: {rejection}")),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_schemas::list_schemas))
        .route("/find", get(find_schema::find_schema))
        .nest(
            "/:schema_id",
            Router::new().route("/", get(get_schema::get_schema)).nest(
                "/variant",
                Router::new()
                    .route("/default", get(get_default_variant::get_default_variant))
                    .nest(
                        "/:schema_variant_id",
                        Router::new().route("/", get(get_variant::get_variant)),
                    ),
            ),
        )
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaVariantV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub variant_id: SchemaVariantId,
    #[schema(value_type = String, example = "AWS EC2 Instance")]
    pub display_name: String,
    #[schema(value_type = String, example = "AWS::EC2")]
    pub category: String,
    #[schema(value_type = String, example = "#FF5733")]
    pub color: String,
    #[schema(value_type = bool, example = false)]
    pub is_locked: bool,
    #[schema(value_type = String, example = "Amazon EC2 Instance resource type")]
    pub description: Option<String>,
    #[schema(value_type = String, example = "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html")]
    pub link: Option<String>,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q75XY")]
    pub asset_func_id: FuncId,
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q75AA", "01H9ZQD35JPMBGHH69BT0Q75BB"]))]
    pub variant_func_ids: Vec<FuncId>,
    #[schema(value_type = bool, example = true)]
    pub is_default_variant: bool,
    #[schema(example = json!({"propId": "01JT71H84S37APM40BJR4KRCVP","name": "domain",
    "propType": "object",
    "description": null,
    "children": [
      {
        "propId": "01JT71H8519R2X9YYP9D7HEFQ9",
        "name": "AdditionalInfo",
        "propType": "string",
        "description": "This property is reserved for internal use. If you use it, the stack fails with this error: Bad property set: [Testing this property] (Service: AmazonEC2; Status Code: 400; Error Code: InvalidParameterCombination; Request ID: 0XXXXXX-49c7-4b40-8bcc-76885dcXXXXX).",
        "children": null
      },
      {
        "propId": "01JT71H8519R2X9YYP9D7HEFQB",
        "name": "Affinity",
        "propType": "string",
        "description": "Indicates whether the instance is associated with a dedicated host. If you want the instance to always restart on the same host on which it was launched, specify host. If you want the instance to restart on any available host, but try to launch onto the last host it ran on (on a best-effort basis), specify default.",
        "children": null
      },
      {
        "propId": "01JT71H8519R2X9YYP9D7HEFQD",
        "name": "AvailabilityZone",
        "propType": "string",
        "description": "The Availability Zone of the instance.",
        "children": null
      },
      {
        "propId": "01JT71H8519R2X9YYP9D7HEFQS",
        "name": "ImageId",
        "propType": "string",
        "description": "The ID of the AMI. An AMI ID is required to launch an instance and must be specified here or in a launch template.",
        "children": null
      },
      {
        "propId": "01JT71H8519R2X9YYP9D7HEFQV",
        "name": "InstanceInitiatedShutdownBehavior",
        "propType": "string",
        "description": "Indicates whether an instance stops or terminates when you initiate shutdown from the instance (using the operating system command for system shutdown).",
        "children": null
      },
      {
        "propId": "01JT71H852NXK3SZC12J625RH1",
        "name": "InstanceType",
        "propType": "string",
        "description": "The instance type.",
        "children": null
      },
      {
        "propId": "01JT71H852NXK3SZC12J625RH7",
        "name": "KeyName",
        "propType": "string",
        "description": "The name of the key pair.",
        "children": null
      },
      {
        "propId": "01JT71H8539E8QEX5JC43N9C6R",
        "name": "SubnetId",
        "propType": "string",
        "description": "[EC2-VPC] The ID of the subnet to launch the instance into.\n\n",
        "children": null
      },
      {
        "propId": "01JT71H8539E8QEX5JC43N9C6T",
        "name": "Tenancy",
        "propType": "string",
        "description": "The tenancy of the instance (if the instance is running in a VPC). An instance with a tenancy of dedicated runs on single-tenant hardware.",
        "children": null
      },
      {
        "propId": "01JT71H8539E8QEX5JC43N9C6W",
        "name": "UserData",
        "propType": "string",
        "description": "The user data to make available to the instance.",
        "children": null
      },
      {
        "propId": "01JT71H854B5QRHBCZY0FG9J1W",
        "name": "CpuOptions",
        "propType": "object",
        "description": "The CPU options for the instance.",
        "children": [
          {
            "propId": "01JT71H854B5QRHBCZY0FG9J20",
            "name": "CoreCount",
            "propType": "number",
            "description": null,
            "children": null
          },
          {
            "propId": "01JT71H854B5QRHBCZY0FG9J22",
            "name": "ThreadsPerCore",
            "propType": "number",
            "description": null,
            "children": null
          }
        ]
      },
      {
        "propId": "01JT71H856YPGMQ2XN88XBJ3SW",
        "name": "Ipv6Addresses",
        "propType": "array",
        "description": "[EC2-VPC] The IPv6 addresses from the range of the subnet to associate with the primary network interface.",
        "children": [
          {
            "propId": "01JT71H856YPGMQ2XN88XBJ3T0",
            "name": "Ipv6AddressesItem",
            "propType": "object",
            "description": null,
            "children": [
              {
                "propId": "01JT71H856YPGMQ2XN88XBJ3T4",
                "name": "Ipv6Address",
                "propType": "string",
                "description": "The IPv6 address.",
                "children": null
              }
            ]
          }
        ]
      },
      {
        "propId": "01JT71H856YPGMQ2XN88XBJ3T6",
        "name": "LaunchTemplate",
        "propType": "object",
        "description": "The launch template to use to launch the instances.",
        "children": [
          {
            "propId": "01JT71H856YPGMQ2XN88XBJ3TA",
            "name": "LaunchTemplateId",
            "propType": "string",
            "description": "The ID of the launch template. You must specify the LaunchTemplateName or the LaunchTemplateId, but not both.",
            "children": null
          },
          {
            "propId": "01JT71H856YPGMQ2XN88XBJ3TC",
            "name": "LaunchTemplateName",
            "propType": "string",
            "description": "The name of the launch template. You must specify the LaunchTemplateName or the LaunchTemplateId, but not both.",
            "children": null
          },
          {
            "propId": "01JT71H856YPGMQ2XN88XBJ3TE",
            "name": "Version",
            "propType": "string",
            "description": "The version number of the launch template.",
            "children": null
          }
        ]
      },
      {
        "propId": "01JT71H85A5HMER65C76PXQCP8",
        "name": "SecurityGroupIds",
        "propType": "array",
        "description": "The IDs of the security groups.",
        "children": [
          {
            "propId": "01JT71H85A5HMER65C76PXQCPC",
            "name": "SecurityGroupIdsItem",
            "propType": "string",
            "description": null,
            "children": null
          }
        ]
      },
      {
        "propId": "01JT71H85A5HMER65C76PXQCPE",
        "name": "SecurityGroups",
        "propType": "array",
        "description": "the names of the security groups. For a nondefault VPC, you must use security group IDs instead.",
        "children": [
          {
            "propId": "01JT71H85A5HMER65C76PXQCPJ",
            "name": "SecurityGroupsItem",
            "propType": "string",
            "description": null,
            "children": null
          }
        ]
      },
      {
        "propId": "01JT71H85B7592MN63P6YNZGZK",
        "name": "Tags",
        "propType": "array",
        "description": "The tags to add to the instance.",
        "children": [
          {
            "propId": "01JT71H85B7592MN63P6YNZGZQ",
            "name": "TagsItem",
            "propType": "object",
            "description": null,
            "children": [
              {
                "propId": "01JT71H85B7592MN63P6YNZGZV",
                "name": "Key",
                "propType": "string",
                "description": null,
                "children": null
              },
              {
                "propId": "01JT71H85B7592MN63P6YNZGZX",
                "name": "Value",
                "propType": "string",
                "description": null,
                "children": null
              }
            ]
          }
        ]
      },
      {
        "propId": "01JT71H85B7592MN63P6YNZGZZ",
        "name": "Volumes",
        "propType": "array",
        "description": "The volumes to attach to the instance.",
        "children": [
          {
            "propId": "01JT71H85B7592MN63P6YNZH03",
            "name": "VolumesItem",
            "propType": "object",
            "description": null,
            "children": [
              {
                "propId": "01JT71H85B7592MN63P6YNZH07",
                "name": "Device",
                "propType": "string",
                "description": "The device name (for example, /dev/sdh or xvdh).",
                "children": null
              },
              {
                "propId": "01JT71H85B7592MN63P6YNZH09",
                "name": "VolumeId",
                "propType": "string",
                "description": "The ID of the EBS volume. The volume and instance must be within the same Availability Zone.",
                "children": null
              }
            ]
          }
        ]
      },
      {
        "propId": "01JT71H85B7592MN63P6YNZH0B",
        "name": "extra",
        "propType": "object",
        "description": null,
        "children": [
          {
            "propId": "01JT71H85B7592MN63P6YNZH0F",
            "name": "Region",
            "propType": "string",
            "description": null,
            "children": null
          },
          {
            "propId": "01JT71H85B7592MN63P6YNZH0H",
            "name": "AwsPermissionsMap",
            "propType": "string",
            "description": null,
            "children": null
          },
          {
            "propId": "01JT71H85B7592MN63P6YNZH0K",
            "name": "AwsResourceType",
            "propType": "string",
            "description": null,
            "children": null
          },
          {
            "propId": "01JT71H85B7592MN63P6YNZH0N",
            "name": "PropUsageMap",
            "propType": "string",
            "description": null,
            "children": null
          }
        ]
      }
    ]}))]
    pub domain_props: Option<PropSchemaV1>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropSchemaV1 {
    #[schema(value_type = String)]
    pub prop_id: PropId,
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = String)]
    pub prop_type: String,
    #[schema(value_type = String)]
    pub description: Option<String>,
    #[schema(value_type = Vec<PropSchemaV1>, no_recursion)]
    pub children: Option<Vec<PropSchemaV1>>,
    // New fields from PropSpecData (excluding func/widget/ui fields)
    #[schema(value_type = String)]
    pub validation_format: Option<String>,
    #[schema(value_type = serde_json::Value)]
    pub default_value: Option<serde_json::Value>,
    #[schema(value_type = bool)]
    pub hidden: Option<bool>,
    #[schema(value_type = String)]
    pub doc_link: Option<String>,
}

impl From<CachedPropSchemaV1> for PropSchemaV1 {
    fn from(cached: CachedPropSchemaV1) -> Self {
        Self {
            prop_id: cached.prop_id,
            name: cached.name,
            prop_type: cached.prop_type,
            description: cached.description,
            children: cached
                .children
                .map(|children| children.into_iter().map(PropSchemaV1::from).collect()),
            validation_format: cached.validation_format,
            default_value: cached.default_value,
            hidden: cached.hidden,
            doc_link: cached.doc_link,
        }
    }
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetSchemaV1Response {
    #[schema(value_type = String, example = "AWS::EC2::Instance")]
    pub name: String,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub default_variant_id: SchemaVariantId,
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79VZ", "01H9ZQD35JPMBGHH69BT0Q79VY"]))]
    pub variant_ids: Vec<SchemaVariantId>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingResponseV1 {
    #[schema(value_type = String, example = "building")]
    pub status: String,
    #[schema(value_type = String, example = "Schema data is being generated, please retry shortly")]
    pub message: String,
    #[schema(value_type = u64, example = 2)]
    pub retry_after_seconds: u64,
    #[schema(value_type = u64, example = 10)]
    pub estimated_completion_seconds: u64,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SchemaVariantResponseV1 {
    Success(Box<GetSchemaVariantV1Response>),
    Building(BuildingResponseV1),
}

impl IntoResponse for SchemaVariantResponseV1 {
    fn into_response(self) -> axum::response::Response {
        match self {
            SchemaVariantResponseV1::Success(response) => {
                (StatusCode::OK, Json(response)).into_response()
            }
            SchemaVariantResponseV1::Building(response) => {
                (StatusCode::ACCEPTED, Json(response)).into_response()
            }
        }
    }
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SchemaResponseV1 {
    Success(GetSchemaV1Response),
    Building(Box<BuildingResponseV1>),
}

impl IntoResponse for SchemaResponseV1 {
    fn into_response(self) -> axum::response::Response {
        match self {
            SchemaResponseV1::Success(response) => (StatusCode::OK, Json(response)).into_response(),
            SchemaResponseV1::Building(response) => {
                (StatusCode::ACCEPTED, Json(response)).into_response()
            }
        }
    }
}
