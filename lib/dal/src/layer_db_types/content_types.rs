use std::collections::{
    HashMap,
    HashSet,
};

use chrono::{
    DateTime,
    Utc,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    CasValue,
    ContentHash,
    Timestamp,
    ulid::Ulid,
};
use strum::EnumDiscriminants;
use thiserror::Error;

use crate::{
    ActionPrototypeId,
    ComponentId,
    ComponentType,
    DalContext,
    FuncBackendKind,
    FuncBackendResponseType,
    FuncId,
    PropId,
    PropKind,
    SchemaVariant,
    SchemaVariantId,
    SocketArity,
    SocketKind,
    UserPk,
    action::{
        ActionCompletionStatus,
        prototype::ActionKind,
    },
    approval_requirement::ApprovalRequirementApprover,
    attribute::path::AttributePath,
    func::argument::FuncArgumentKind,
    prop::WidgetOptions,
    property_editor::schema::WidgetKind,
    socket::connection_annotation::ConnectionAnnotation,
    validation::ValidationStatus,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ContentTypeError {
    #[error("error extracting schema variant content : {0}")]
    SchemaVariantContent(String),
}

pub type ContentTypeResult<T> = Result<T, ContentTypeError>;

/// This type gathers all the kinds of things we will store in the
/// content-store portion of the layered database. Anything we want to read or
/// write from there should be added here. Then the impl_into_content_types!
/// macro should be used to provide from/into implementations between the inner
/// type and this enum. The naming pattern here should ALWAYS be observed (with
/// the exception of `Any(CasValue)`, since the macro's implementation depends
/// on it. (If you want to break the convention you have to write your own
/// `From` implementations).  To ensure we don't break the enum deserialization
/// with postcard, DO *NOT* add new types to this list in alphabetical order.
/// Add them to the *END* of the enum *ONLY*.
#[derive(EnumDiscriminants, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[strum_discriminants(derive(strum::EnumIter))]
pub enum ContentTypes {
    Any(CasValue),
    AttributePrototype(AttributePrototypeContent),
    Component(ComponentContent),
    DeprecatedAction(DeprecatedActionContent),
    DeprecatedActionBatch(DeprecatedActionBatchContent),
    DeprecatedActionPrototype(DeprecatedActionPrototypeContent),
    DeprecatedActionRunner(DeprecatedActionRunnerContent),
    Func(FuncContent),
    FuncArgument(FuncArgumentContent),
    InputSocket(InputSocketContent),
    Module(ModuleContent),
    Prop(PropContent),
    Schema(SchemaContent),
    SchemaVariant(SchemaVariantContent),
    Secret(SecretContent),
    StaticArgumentValue(StaticArgumentValueContent),
    Validation(ValidationContent),
    OutputSocket(OutputSocketContent),
    ManagementPrototype(ManagementPrototypeContent),
    Geometry(GeometryContent),
    View(ViewContent),
    ApprovalRequirementDefinition(ApprovalRequirementDefinitionContent),
    AttributePaths(AttributePathsContent),
}

macro_rules! impl_into_content_types {
    (
        $(#[$($attrss:tt)*])*
        $name:ident
    ) => {
        paste::paste! {
            impl From<[<$name Content>]> for ContentTypes {
                fn from(value: [<$name Content>]) -> Self {
                    ContentTypes::$name(value)
                }
            }

            impl TryFrom<ContentTypes> for [<$name Content>] {
                type Error = &'static str;

                fn try_from(value: ContentTypes) -> Result<Self, Self::Error> {
                    match value {
                        ContentTypes::$name(inner) => Ok(inner),
                        _ => Err(std::concat!("Could not convert ContentType to ", stringify!($name)))
                    }
                }
            }

            impl From<ContentTypes> for Option<[<$name Content>]> {
                fn from(value: ContentTypes) -> Self {
                    match value {
                        ContentTypes::$name(inner) => Some(inner),
                        _ => None
                    }
                }
            }
        }
    };
}

impl_into_content_types!(DeprecatedActionPrototype);
impl_into_content_types!(AttributePrototype);
impl_into_content_types!(Component);
impl_into_content_types!(DeprecatedAction);
impl_into_content_types!(DeprecatedActionBatch);
impl_into_content_types!(DeprecatedActionRunner);
impl_into_content_types!(Func);
impl_into_content_types!(FuncArgument);
impl_into_content_types!(InputSocket);
impl_into_content_types!(OutputSocket);
impl_into_content_types!(Module);
impl_into_content_types!(Prop);
impl_into_content_types!(Schema);
impl_into_content_types!(SchemaVariant);
impl_into_content_types!(Secret);
impl_into_content_types!(StaticArgumentValue);
impl_into_content_types!(Validation);
impl_into_content_types!(ManagementPrototype);
impl_into_content_types!(Geometry);
impl_into_content_types!(View);
impl_into_content_types!(ApprovalRequirementDefinition);

// Here we've broken the Foo, FooContent convention so we need to implement
// these traits manually
impl From<CasValue> for ContentTypes {
    fn from(value: CasValue) -> Self {
        ContentTypes::Any(value)
    }
}

impl TryFrom<ContentTypes> for CasValue {
    type Error = &'static str;

    fn try_from(value: ContentTypes) -> Result<Self, Self::Error> {
        match value {
            ContentTypes::Any(inner) => Ok(inner),
            _ => Err("Could not convert ContentType to CasValue"),
        }
    }
}

impl From<ContentTypes> for Option<CasValue> {
    fn from(value: ContentTypes) -> Self {
        match value {
            ContentTypes::Any(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum DeprecatedActionContent {
    V1(DeprecatedActionContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeprecatedActionContentV1 {
    pub creation_user_pk: Option<UserPk>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum DeprecatedActionBatchContent {
    V1(DeprecatedActionBatchContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeprecatedActionBatchContentV1 {
    pub author: String,
    pub actors: String,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub completion_status: Option<ActionCompletionStatus>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum DeprecatedActionPrototypeContent {
    V1(DeprecatedActionPrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeprecatedActionPrototypeContentV1 {
    pub kind: ActionKind,
    pub name: Option<String>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum DeprecatedActionRunnerContent {
    V1(DeprecatedActionRunnerContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct DeprecatedActionRunnerContentV1 {
    pub timestamp: Timestamp,

    pub component_id: ComponentId,
    pub component_name: String,
    pub schema_name: String,
    pub func_name: String,
    pub action_prototype_id: ActionPrototypeId,
    pub action_kind: ActionKind,
    pub resource: Option<String>,

    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub completion_status: Option<ActionCompletionStatus>,
    pub completion_message: Option<String>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum AttributePrototypeContent {
    V1(AttributePrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributePrototypeContentV1 {
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
#[strum_discriminants(derive(strum::Display))]
pub enum ComponentContent {
    V1(ComponentContentV1),
    V2(ComponentContentV2),
}

impl ComponentContent {
    pub fn extract(self) -> ComponentContentV2 {
        match self {
            ComponentContent::V1(v1) => ComponentContentV2 {
                timestamp: v1.timestamp,
            },
            ComponentContent::V2(v2) => v2,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ComponentContentV1 {
    pub timestamp: Timestamp,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ComponentContentV2 {
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ViewContent {
    V1(ViewContentV1),
}

impl ViewContent {
    pub fn extract(self) -> ViewContentV1 {
        let ViewContent::V1(content) = self;
        content
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ViewContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum GeometryContent {
    V1(GeometryContentV1),
}

impl GeometryContent {
    pub fn extract(self) -> GeometryContentV1 {
        let GeometryContent::V1(content) = self;
        content
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct GeometryContentV1 {
    pub timestamp: Timestamp,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum FuncContent {
    V1(FuncContentV1),
    V2(FuncContentV2),
    V3(FuncContentV3),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FuncContentV1 {
    pub timestamp: Timestamp,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub hidden: bool,
    pub builtin: bool,
    pub backend_response_type: FuncBackendResponseType,
    pub backend_kind: FuncBackendKind,
    pub handler: Option<String>,
    pub code_base64: Option<String>,
    /// A hash of the code above
    pub code_blake3: ContentHash,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FuncContentV2 {
    pub timestamp: Timestamp,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub hidden: bool,
    pub builtin: bool,
    pub backend_response_type: FuncBackendResponseType,
    pub backend_kind: FuncBackendKind,
    pub handler: Option<String>,
    pub code_base64: Option<String>,
    /// A hash of the code above
    pub code_blake3: ContentHash,
    pub is_locked: bool,
}
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct FuncContentV3 {
    pub timestamp: Timestamp,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub hidden: bool,
    pub builtin: bool,
    pub backend_response_type: FuncBackendResponseType,
    pub backend_kind: FuncBackendKind,
    pub handler: Option<String>,
    pub code_base64: Option<String>,
    /// A hash of the code above
    pub code_blake3: ContentHash,
    pub is_locked: bool,
    // Transformation funcs always have a single argument, and aren't created in the context of a schema variant
    pub is_transformation: bool,
}

impl FuncContent {
    pub fn extract(self) -> FuncContentV3 {
        match self {
            FuncContent::V1(v1) => FuncContentV3 {
                timestamp: v1.timestamp,
                hidden: v1.hidden,
                display_name: v1.display_name,
                link: v1.link,
                description: v1.description,
                is_locked: true,
                builtin: v1.builtin,
                backend_response_type: v1.backend_response_type,
                backend_kind: v1.backend_kind,
                handler: v1.handler,
                code_base64: v1.code_base64,
                code_blake3: v1.code_blake3,
                is_transformation: false,
            },
            FuncContent::V2(v1) => FuncContentV3 {
                timestamp: v1.timestamp,
                hidden: v1.hidden,
                display_name: v1.display_name,
                link: v1.link,
                description: v1.description,
                is_locked: true,
                builtin: v1.builtin,
                backend_response_type: v1.backend_response_type,
                backend_kind: v1.backend_kind,
                handler: v1.handler,
                code_base64: v1.code_base64,
                code_blake3: v1.code_blake3,
                is_transformation: false,
            },
            FuncContent::V3(v3) => v3,
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum FuncArgumentContent {
    V1(FuncArgumentContentV1),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FuncArgumentContentV1 {
    pub kind: FuncArgumentKind,
    pub element_kind: Option<FuncArgumentKind>,
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum InputSocketContent {
    V1(InputSocketContentV1),
    V2(InputSocketContentV2),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InputSocketContentV1 {
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    pub inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    pub outbound_type_definition: Option<String>,
    pub arity: SocketArity,
    pub kind: SocketKind,
    pub required: bool,
    pub ui_hidden: bool,
    pub connection_annotations: Vec<ConnectionAnnotation>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InputSocketContentV2 {
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the inbound type (e.g. "JSONSchema" or "Number").
    pub inbound_type_definition: Option<String>,
    /// Definition of the outbound type (e.g. "JSONSchema" or "Number").
    pub outbound_type_definition: Option<String>,
    pub kind: SocketKind,
    pub required: bool,
    pub ui_hidden: bool,
    pub connection_annotations: Vec<ConnectionAnnotation>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ModuleContent {
    V1(ModuleContentV1),
    V2(ModuleContentV2),
}

impl ModuleContent {
    pub fn inner(&self) -> ModuleContentV2 {
        match self {
            ModuleContent::V1(inner) => inner.to_owned().into(),
            ModuleContent::V2(inner) => inner.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModuleContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
    pub root_hash: String,
    pub version: String,
    pub description: String,
    pub created_by_email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ModuleContentV2 {
    pub timestamp: Timestamp,
    pub name: String,
    pub root_hash: String,
    pub version: String,
    pub description: String,
    pub created_by_email: String,
    pub created_at: DateTime<Utc>,
    pub schema_id: Option<Ulid>,
}

impl From<ModuleContentV1> for ModuleContentV2 {
    fn from(value: ModuleContentV1) -> Self {
        Self {
            timestamp: value.timestamp,
            name: value.name,
            root_hash: value.root_hash,
            version: value.version,
            description: value.description,
            created_by_email: value.created_by_email,
            created_at: value.created_at,
            schema_id: None,
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum OutputSocketContent {
    V1(OutputSocketContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct OutputSocketContentV1 {
    pub timestamp: Timestamp,
    /// Name for [`Self`] that can be used for identification.
    pub name: String,
    /// Definition of the data type (e.g. "JSONSchema" or "Number").
    pub type_definition: Option<String>,
    pub arity: SocketArity,
    pub kind: SocketKind,
    pub required: bool,
    pub ui_hidden: bool,
    pub connection_annotations: Vec<ConnectionAnnotation>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq, derive_more::From)]
pub enum PropContent {
    V1(PropContentV1),
    V2(PropContentV2),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PropContentV1 {
    pub timestamp: Timestamp,
    /// The name of the [`Prop`].
    pub name: String,
    /// The kind of the [`Prop`].
    pub kind: PropKind,
    /// The kind of "widget" that should be used for this [`Prop`].
    pub widget_kind: WidgetKind,
    /// The configuration of the "widget".
    pub widget_options: Option<WidgetOptions>,
    /// A link to external documentation for working with this specific [`Prop`].
    pub doc_link: Option<String>,
    /// Embedded documentation for working with this specific [`Prop`].
    pub documentation: Option<String>,
    /// A toggle for whether or not the [`Prop`] should be visually hidden.
    pub hidden: bool,
    /// Props can be connected to eachother to signify that they should contain the same value
    /// This is useful for diffing the resource with the domain, to suggest actions if the real world changes
    pub refers_to_prop_id: Option<PropId>,
    /// Connected props may need a custom diff function
    pub diff_func_id: Option<FuncId>,
    /// A serialized validation format JSON object for the prop.
    pub validation_format: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PropContentV2 {
    pub timestamp: Timestamp,
    /// The name of the [`Prop`].
    pub name: String,
    /// The kind of the [`Prop`].
    pub kind: PropKind,
    /// The kind of "widget" that should be used for this [`Prop`].
    pub widget_kind: WidgetKind,
    /// The configuration of the "widget".
    pub widget_options: Option<WidgetOptions>,
    /// A link to external documentation for working with this specific [`Prop`].
    pub doc_link: Option<String>,
    /// Embedded documentation for working with this specific [`Prop`].
    pub documentation: Option<String>,
    /// A toggle for whether or not the [`Prop`] should be visually hidden.
    pub hidden: bool,
    /// Props can be connected to eachother to signify that they should contain the same value
    /// This is useful for diffing the resource with the domain, to suggest actions if the real world changes
    pub refers_to_prop_id: Option<PropId>,
    /// Connected props may need a custom diff function
    pub diff_func_id: Option<FuncId>,
    /// A serialized validation format JSON object for the prop.
    pub validation_format: Option<String>,
    /// Optional UI data that isn't needed in the graph
    /// (Eventually, many other fields in content will be moved here, e.g. documentation/docLink)
    pub ui_optionals: Option<HashMap<String, CasValue>>,
}

impl From<PropContentV1> for PropContentV2 {
    fn from(
        PropContentV1 {
            timestamp,
            name,
            kind,
            widget_kind,
            widget_options,
            doc_link,
            documentation,
            hidden,
            refers_to_prop_id,
            diff_func_id,
            validation_format,
        }: PropContentV1,
    ) -> Self {
        Self {
            timestamp,
            name,
            kind,
            widget_kind,
            widget_options,
            doc_link,
            documentation,
            hidden,
            refers_to_prop_id,
            diff_func_id,
            validation_format,
            ui_optionals: None, // This field is not present in V1
        }
    }
}

impl From<PropContent> for PropContentV2 {
    fn from(value: PropContent) -> Self {
        match value {
            PropContent::V1(v1) => v1.into(),
            PropContent::V2(v2) => v2,
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SchemaContent {
    V1(SchemaContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
    pub ui_hidden: bool,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SchemaVariantContent {
    V1(SchemaVariantContentV1),
    V2(SchemaVariantContentV2),
    V3(SchemaVariantContentV3),
}

impl SchemaVariantContent {
    pub async fn extract(
        self,
        ctx: &DalContext,
        id: SchemaVariantId,
    ) -> ContentTypeResult<SchemaVariantContentV3> {
        // update progressively
        let mut working_content = self;
        loop {
            working_content = match working_content {
                SchemaVariantContent::V1(v1) => {
                    let display_name = if let Some(display_name) = v1.display_name {
                        display_name
                    } else {
                        let schema = SchemaVariant::schema_for_schema_variant_id(ctx, id)
                            .await
                            .map_err(|e| ContentTypeError::SchemaVariantContent(e.to_string()))?;
                        schema.name
                    };

                    SchemaVariantContent::V2(SchemaVariantContentV2 {
                        timestamp: v1.timestamp,
                        ui_hidden: v1.ui_hidden,
                        version: v1.name.to_owned(),
                        display_name,
                        category: v1.category,
                        color: v1.color,
                        component_type: v1.component_type,
                        link: v1.link,
                        description: v1.description,
                        asset_func_id: v1.asset_func_id,
                        finalized_once: v1.finalized_once,
                        is_builtin: v1.is_builtin,
                        is_locked: true,
                    })
                }
                SchemaVariantContent::V2(v2) => SchemaVariantContent::V3(SchemaVariantContentV3 {
                    timestamp: v2.timestamp,
                    ui_hidden: v2.ui_hidden,
                    version: v2.version,
                    display_name: v2.display_name,
                    category: v2.category,
                    color: v2.color,
                    component_type: v2.component_type,
                    link: v2.link,
                    description: v2.description,
                    asset_func_id: v2.asset_func_id,
                    finalized_once: v2.finalized_once,
                    is_builtin: v2.is_builtin,
                }),
                SchemaVariantContent::V3(_) => break,
            };
        }

        // extract latest
        let latest = match working_content {
            SchemaVariantContent::V3(v3) => v3,
            _ => unreachable!(),
        };

        Ok(latest)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaVariantContentV1 {
    pub timestamp: Timestamp,
    pub ui_hidden: bool,
    pub name: String,
    pub display_name: Option<String>,
    pub category: String,
    pub color: String,
    pub component_type: ComponentType,
    pub link: Option<String>,
    pub description: Option<String>,
    pub asset_func_id: Option<FuncId>,
    pub finalized_once: bool,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaVariantContentV2 {
    pub timestamp: Timestamp,
    pub ui_hidden: bool,
    pub version: String,
    pub display_name: String,
    pub category: String,
    pub color: String,
    pub component_type: ComponentType,
    pub link: Option<String>,
    pub description: Option<String>,
    pub asset_func_id: Option<FuncId>,
    pub finalized_once: bool,
    pub is_builtin: bool,
    pub is_locked: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaVariantContentV3 {
    pub timestamp: Timestamp,
    pub ui_hidden: bool,
    pub version: String,
    pub display_name: String,
    pub category: String,
    pub color: String,
    pub component_type: ComponentType,
    pub link: Option<String>,
    pub description: Option<String>,
    pub asset_func_id: Option<FuncId>,
    pub finalized_once: bool,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SecretContent {
    V1(SecretContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SecretContentV1 {
    pub timestamp: Timestamp,
    pub created_by: Option<UserPk>,
    pub updated_by: Option<UserPk>,

    pub name: String,
    pub definition: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum StaticArgumentValueContent {
    V1(StaticArgumentValueContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct StaticArgumentValueContentV1 {
    pub timestamp: Timestamp,
    pub value: si_events::CasValue,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ValidationContent {
    V1(ValidationContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ValidationContentV1 {
    pub timestamp: Timestamp,
    pub status: ValidationStatus,
    pub message: Option<String>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ManagementPrototypeContent {
    V1(ManagementPrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ManagementPrototypeContentV1 {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ApprovalRequirementDefinitionContent {
    V1(ApprovalRequirementDefinitionContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ApprovalRequirementDefinitionContentV1 {
    pub minimum: usize,
    pub approvers: HashSet<ApprovalRequirementApprover>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum AttributePathsContent {
    V1(AttributePathsContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct AttributePathsContentV1(pub Vec<AttributePath>);

impl From<Vec<AttributePath>> for AttributePathsContent {
    fn from(paths: Vec<AttributePath>) -> Self {
        AttributePathsContent::V1(AttributePathsContentV1(paths))
    }
}
