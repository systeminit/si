use std::collections::{
    HashMap,
    HashSet,
};

use axum::{
    Json,
    Router,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::IntoResponse,
    routing::{
        delete,
        get,
        post,
        put,
    },
};
use dal::{
    DalContext,
    FuncError,
    FuncId,
    PropId,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    TransactionsError,
    action::prototype::{
        ActionKind,
        ActionPrototypeError,
    },
    cached_module::CachedModule,
    func::{
        FuncKind,
        authoring::FuncAuthoringError,
        binding::FuncBindingError,
    },
    management::prototype::ManagementPrototypeError,
    prop::PropError,
    schema::{
        leaf::LeafPrototypeError,
        variant::authoring::VariantAuthoringError,
    },
};
use frigg::FriggError;
use serde::{
    Deserialize,
    Serialize,
};
use si_frontend_mv_types::{
    luminork_schema_variant_func::{
        FuncKindVariant,
        LuminorkSchemaVariantFunc,
    },
    management::ManagementFuncKind,
    prop_schema::PropSchemaV1 as CachedPropSchemaV1,
};
use telemetry::prelude::*;
use telemetry_utils::monotonic;
use thiserror::Error;
use utoipa::{
    self,
    ToSchema,
};

use crate::AppState;

pub mod contribute;
pub mod create_action;
pub mod create_attribute;
pub mod create_authentication;
pub mod create_codegen;
pub mod create_management;
pub mod create_qualification;
pub mod create_schema;
pub mod detach_action_binding;
pub mod detach_attribute_binding;
pub mod detach_authentication_binding;
pub mod detach_codegen_binding;
pub mod detach_management_binding;
pub mod detach_qualification_binding;
pub mod find_schema;
pub mod get_default_variant;
pub mod get_schema;
pub mod get_variant;
pub mod install_schema;
pub mod list_schemas;
pub mod search_schemas;
pub mod unlock_schema;
pub mod update_schema_variant;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SchemaError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] dal::attribute::prototype::AttributePrototypeError),
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("cannot contribute unlocked schema variant: {0}")]
    ContributeUnlockedVariant(SchemaVariantId),
    #[error("cannot contribute builtin schema: {0}")]
    ContributeUpstreamSchema(SchemaId),
    #[error("contributions can only be made on HEAD")]
    ContributionsMustBeMadeFromHead,
    #[error("decode error: {0}")]
    Decode(#[from] ulid::DecodeError),
    #[error("frigg error: {0}")]
    Frigg(#[from] FriggError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] dal::func::argument::FuncArgumentError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("leaf prototype error: {0}")]
    LeafPrototype(#[from] LeafPrototypeError),
    #[error("trying to modify locked variant: {0}")]
    LockedVariant(SchemaVariantId),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] ManagementPrototypeError),
    #[error("materialized views error: {0}")]
    MaterializedViews(#[from] dal_materialized_views::Error),
    #[error("missing input location for attribute func argument")]
    MissingInputLocationForAttributeFunc,
    #[error("missing output location (prop_id or output_socket_id) for attribute func")]
    MissingOutputLocationForAttributeFunc,
    #[error("schema missing asset func id: {0}")]
    MissingVariantFunc(SchemaVariantId),
    #[error("module error: {0}")]
    Module(#[from] dal::module::ModuleError),
    #[error("module index client error: {0}")]
    ModuleIndexClient(#[from] module_index_client::ModuleIndexClientError),
    #[error("module index not configured")]
    ModuleIndexNotConfigured,
    #[error("changes not permitted on HEAD change set")]
    NotPermittedOnHead,
    #[error("output socket error: {0}")]
    OutputSocket(#[from] dal::socket::output::OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema not found error: {0}")]
    SchemaNotFound(SchemaId),
    #[error("schema not found by name: {0}")]
    SchemaNotFoundByName(String),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("schema variant not found error: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("schema variant {0} not a variant for the schema {1} error")]
    SchemaVariantNotMemberOfSchema(SchemaId, SchemaVariantId),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] dal::slow_rt::SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("url parse error: {0}")]
    UrlParse(#[from] url::ParseError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
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

#[derive(Deserialize, ToSchema)]
pub struct SchemaVariantFuncV1RequestPath {
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String)]
    pub schema_variant_id: SchemaVariantId,
    #[schema(value_type = String)]
    pub func_id: FuncId,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct DetachFuncBindingV1Response {
    #[schema(example = true)]
    pub success: bool,
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
            SchemaError::SchemaNotFoundByName(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaVariantNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            SchemaError::SchemaVariantNotMemberOfSchema(_, _) => {
                (StatusCode::PRECONDITION_REQUIRED, self.to_string())
            }
            SchemaError::ContributionsMustBeMadeFromHead => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            SchemaError::ContributeUpstreamSchema(_) => {
                (StatusCode::PRECONDITION_FAILED, self.to_string())
            }
            SchemaError::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, self.to_string()),
            SchemaError::SchemaVariant(dal::SchemaVariantError::SchemaVariantLocked(_)) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
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
        .route("/", post(create_schema::create_schema))
        .route("/find", get(find_schema::find_schema))
        .route("/search", post(search_schemas::search_schemas))
        .nest(
            "/:schema_id",
            Router::new()
                .route("/", get(get_schema::get_schema))
                .route("/unlock", post(unlock_schema::unlock_schema))
                .route("/install", post(install_schema::install_schema))
                .route("/contribute", post(contribute::contribute))
                .nest(
                    "/variant",
                    Router::new()
                        .route("/default", get(get_default_variant::get_default_variant))
                        .nest(
                            "/:schema_variant_id",
                            Router::new()
                                .route("/", get(get_variant::get_variant))
                                .route("/", put(update_schema_variant::update_schema_variant))
                                .nest(
                                "/funcs",
                                Router::new()
                                    .route("/action", post(create_action::create_variant_action))
                                    .route(
                                        "/action/:func_id",
                                        delete(detach_action_binding::detach_action_func_binding),
                                    )
                                    .route(
                                        "/management",
                                        post(create_management::create_variant_management),
                                    )
                                    .route(
                                        "/management/:func_id",
                                        delete(detach_management_binding::detach_management_func_binding),
                                    )
                                    .route(
                                        "/authentication",
                                        post(create_authentication::create_variant_authentication),
                                    )
                                    .route(
                                        "/authentication/:func_id",
                                        delete(detach_authentication_binding::detach_authentication_func_binding),
                                    )
                                    .route("/codegen", post(create_codegen::create_variant_codegen))
                                    .route(
                                        "/codegen/:func_id",
                                        delete(detach_codegen_binding::detach_codegen_func_binding),
                                    )
                                    .route(
                                        "/qualification",
                                        post(create_qualification::create_variant_qualification),
                                    )
                                    .route(
                                        "/qualification/:func_id",
                                        delete(detach_qualification_binding::detach_qualification_func_binding),
                                    )
                                    .route(
                                        "/attribute",
                                        post(create_attribute::create_variant_attribute),
                                    )
                                    .route(
                                        "/attribute/:func_id",
                                        delete(detach_attribute_binding::detach_attribute_func_binding),
                                    ),
                            ),
                        ),
                ),
        )
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantFunc {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub id: FuncId,
    pub func_kind: SchemaVariantFuncKind,
    pub is_overlay: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, ToSchema)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SchemaVariantFuncKind {
    /// Action function; carries the specific `ActionKind`.
    #[serde(rename_all = "camelCase")]
    Action {
        /// Specific action kind
        #[schema(value_type = String, example = "Create")]
        action_kind: ActionKind,
    },

    /// Management function; carries the specific `ManagementFuncKind`.
    #[serde(rename_all = "camelCase")]
    Management {
        /// Specific management function kind
        #[schema(value_type = String, example = "Import")]
        management_func_kind: ManagementFuncKind,
    },

    /// Any other function; exposes the raw `FuncKind` category.
    #[serde(rename_all = "camelCase")]
    Other {
        #[schema(value_type = String, example = "Qualification")]
        func_kind: FuncKind,
    },
}

impl From<LuminorkSchemaVariantFunc> for SchemaVariantFunc {
    fn from(inner: LuminorkSchemaVariantFunc) -> Self {
        SchemaVariantFunc {
            id: inner.id,
            func_kind: match inner.func_kind {
                FuncKindVariant::Action(k) => SchemaVariantFuncKind::Action {
                    action_kind: k.into(),
                },
                FuncKindVariant::Management(k) => SchemaVariantFuncKind::Management {
                    management_func_kind: k,
                },
                FuncKindVariant::Other(k) => SchemaVariantFuncKind::Other {
                    func_kind: k.into(),
                },
            },
            is_overlay: inner.is_overlay,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
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
    #[schema(value_type = bool, example = false)]
    pub installed_from_upstream: bool,
    #[schema(value_type = Option<String>, example = "Amazon EC2 Instance resource type")]
    pub description: Option<String>,
    #[schema(value_type = Option<String>, example = "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide/aws-resource-ec2-instance.html")]
    pub link: Option<String>,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q75XY")]
    pub asset_func_id: FuncId,
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q75AA", "01H9ZQD35JPMBGHH69BT0Q75BB"]))]
    #[deprecated(
        note = "variant_func_ids deprecated in favour of using variant_funcs parameter and will be removed in a future version of the API"
    )]
    pub variant_func_ids: Vec<FuncId>,
    pub variant_funcs: Vec<SchemaVariantFunc>,
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

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropSchemaV1 {
    #[schema(value_type = String)]
    pub prop_id: PropId,
    #[schema(value_type = String)]
    pub name: String,
    #[schema(value_type = String)]
    pub prop_type: String,
    #[schema(value_type = Option<String>)]
    pub description: Option<String>,
    #[schema(no_recursion)]
    pub children: Option<Vec<PropSchemaV1>>,
    // New fields from PropSpecData (excluding func/widget/ui fields)
    #[schema(value_type = Option<String>)]
    pub validation_format: Option<String>,
    pub default_value: Option<serde_json::Value>,
    #[schema(value_type = Option<bool>)]
    pub hidden: Option<bool>,
    #[schema(value_type = Option<String>)]
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
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String, example = "AWS::EC2::Instance")]
    pub name: String,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VZ")]
    pub default_variant_id: SchemaVariantId,
    #[schema(value_type = Vec<String>, example = json!(["01H9ZQD35JPMBGHH69BT0Q79VZ", "01H9ZQD35JPMBGHH69BT0Q79VY"]))]
    pub variant_ids: Vec<SchemaVariantId>,
}

/// The response payload when materialized views or data is being built referenced by present or
/// expected data.
#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingResponseV1 {
    /// The status of the data being built.
    #[schema(value_type = String, example = "building")]
    pub status: String,
    /// The message reflecting the reason or state of the data being built.
    #[schema(value_type = String, example = "Schema data is being generated, please retry shortly")]
    pub message: String,
    /// The number of seconds recommended between retries for the desired data.
    #[schema(value_type = u64, example = 2)]
    pub retry_after_seconds: u64,
    /// The estimated time for the data being built to be completed.
    #[schema(value_type = u64, example = 10)]
    pub estimated_completion_seconds: u64,
}

impl BuildingResponseV1 {
    /// Creates a new response and increments the relevant counter for default variant data
    /// generated from the workspace graph.
    pub fn new_and_increment_counter_for_default_variant_workspace_graph() -> Self {
        monotonic!(luminork_building_default_variant_workspace_graph = 1);
        Self::new_inner(
            "Default variant data is being generated from workspace graph, please retry shortly",
            2,
            5,
        )
    }

    /// Creates a new response and increments the relevant counter for default variant data
    /// generated from cached modules.
    pub fn new_and_increment_counter_for_default_variant_cached_modules() -> Self {
        monotonic!(luminork_building_default_variant_cached_modules = 1);
        Self::new_inner(
            "Default variant data is being generated from cached modules, please retry shortly",
            2,
            10,
        )
    }

    /// Creates a new response and increments the relevant counter for schema data
    /// generated from cached modules.
    pub fn new_and_increment_counter_for_schema_cached_modules() -> Self {
        monotonic!(luminork_building_schema_cached_modules = 1);
        Self::new_inner(
            "Schema data is being generated from cached modules, please retry shortly",
            2,
            10,
        )
    }

    /// Creates a new response and increments the relevant counter for schema variant data
    /// generated from the workspace graph.
    pub fn new_and_increment_counter_for_schema_variant_workspace_graph() -> Self {
        monotonic!(luminork_building_schema_variant_workspace_graph = 1);
        Self::new_inner(
            "Schema variant data is being generated from workspace graph, please retry shortly",
            2,
            5,
        )
    }

    /// Creates a new response and increments the relevant counter for schema data
    /// generated from cached modules.
    pub fn new_and_increment_counter_for_schema_variant_cached_modules() -> Self {
        monotonic!(luminork_building_schema_variant_cached_modules = 1);
        Self::new_inner(
            "Schema variant data is being generated from cached modules, please retry shortly",
            2,
            10,
        )
    }

    fn new_inner(
        message: &'static str,
        retry_after_seconds: u64,
        estimated_completion_seconds: u64,
    ) -> Self {
        Self {
            status: "building".to_string(),
            message: message.to_string(),
            retry_after_seconds,
            estimated_completion_seconds,
        }
    }
}

#[derive(Debug, Serialize)]
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

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SchemaResponse {
    #[schema(example = "AWS::EC2::Instance")]
    pub schema_name: String,
    #[schema(value_type = Option<String>, example = "AWS::EC2")]
    pub category: Option<String>,
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub schema_id: SchemaId,
    #[schema(value_type = bool, example = "false")]
    pub installed: bool,
}

pub async fn get_full_schema_list(ctx: &DalContext) -> SchemaResult<Vec<SchemaResponse>> {
    let schema_ids = dal::Schema::list_ids(ctx).await?;
    let installed_schema_ids: HashSet<_> = schema_ids.iter().collect();

    // Get cached modules with their metadata
    let cached_modules = CachedModule::latest_modules(ctx).await?;
    // Create a map of schema ID to cached module data
    let mut cached_module_map: HashMap<SchemaId, CachedModule> = HashMap::new();
    for module in cached_modules {
        cached_module_map.insert(module.schema_id, module);
    }

    // Combine both sources to create a complete list
    let mut all_schemas: Vec<SchemaResponse> = Vec::new();
    // First add installed schemas from Schema::list_ids
    for schema_id in &schema_ids {
        if let Some(module) = cached_module_map.get(schema_id) {
            // Schema is both installed and in cache
            all_schemas.push(SchemaResponse {
                schema_name: module.schema_name.clone(),
                schema_id: *schema_id,
                category: module.category.clone(),
                installed: true,
            });
        } else {
            // Schema is installed but not in cache - this is a local only schema
            if let Ok(schema) = dal::Schema::get_by_id(ctx, *schema_id).await {
                let default_variant = SchemaVariant::default_for_schema(ctx, *schema_id).await?;
                all_schemas.push(SchemaResponse {
                    schema_name: schema.name,
                    schema_id: *schema_id,
                    category: Some(default_variant.category().to_owned()),
                    installed: true,
                });
            }
        }

        cached_module_map.remove(schema_id);
    }

    // Now add remaining cached modules (uninstalled ones)
    for (schema_id, module) in cached_module_map {
        let is_installed = installed_schema_ids.contains(&schema_id);
        all_schemas.push(SchemaResponse {
            schema_name: module.schema_name,
            schema_id,
            category: module.category,
            installed: is_installed,
        });
    }

    Ok(all_schemas)
}
