use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_events::{CasValue, ContentHash, EncryptedSecretKey};
use strum::EnumDiscriminants;

use crate::{
    func::argument::FuncArgumentKind, prop::WidgetOptions, property_editor::schema::WidgetKind,
    socket::connection_annotation::ConnectionAnnotation, ActionCompletionStatus, ActionKind,
    ActionPrototypeId, ComponentId, ComponentType, FuncBackendKind, FuncBackendResponseType,
    FuncId, PropId, PropKind, SocketArity, SocketKind, Timestamp, UserPk,
};

/// This type gathers up all the kinds of things we will store in the
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
pub enum ContentTypes {
    ActionPrototype(ActionPrototypeContent),
    Any(CasValue),
    AttributePrototype(AttributePrototypeContent),
    Component(ComponentContent),
    DeprecatedAction(DeprecatedActionContent),
    DeprecatedActionBatch(DeprecatedActionBatchContent),
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
    OutputSocket(OutputSocketContent),
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

impl_into_content_types!(ActionPrototype);
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
pub enum ActionPrototypeContent {
    V1(ActionPrototypeContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ActionPrototypeContentV1 {
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
pub enum ComponentContent {
    V1(ComponentContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ComponentContentV1 {
    pub timestamp: Timestamp,
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum FuncContent {
    V1(FuncContentV1),
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

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum ModuleContent {
    V1(ModuleContentV1),
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

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum PropContent {
    V1(PropContentV1),
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

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SchemaContent {
    V1(SchemaContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SchemaContentV1 {
    pub timestamp: Timestamp,
    pub name: String,
    pub ui_hidden: bool,
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SchemaVariantContent {
    V1(SchemaVariantContentV1),
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
}

#[derive(Debug, Clone, EnumDiscriminants, Serialize, Deserialize, PartialEq)]
pub enum SecretContent {
    V1(SecretContentV1),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SecretContentV1 {
    pub key: EncryptedSecretKey,

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
