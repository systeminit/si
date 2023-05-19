use std::convert::TryFrom;
use thiserror::Error;
use url::ParseError;

mod export;
mod import;

pub use export::export_pkg;
pub use export::get_component_type;
pub use import::{import_pkg, import_pkg_from_pkg};

use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};

use crate::{
    func::{
        argument::{FuncArgumentError, FuncArgumentId},
        binding::FuncBindingError,
    },
    installed_pkg::InstalledPkgError,
    prop_tree::PropTreeError,
    schema::variant::definition::SchemaVariantDefinitionError,
    socket::SocketError,
    ActionPrototypeError, AttributeContextBuilderError, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId, AttributePrototypeError, AttributePrototypeId,
    AttributeReadContext, AttributeValueError, ExternalProviderError, ExternalProviderId,
    FuncBackendKind, FuncBackendResponseType, FuncError, FuncId, InternalProviderError,
    InternalProviderId, PropError, PropId, PropKind, SchemaError, SchemaId, SchemaVariantError,
    SchemaVariantId, StandardModelError, ValidationPrototypeError,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PkgError {
    #[error("Action creation error: {0}")]
    Action(#[from] ActionPrototypeError),
    #[error(transparent)]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute function for context {0:?} has key {1} but is not setting a prop value")]
    AttributeFuncForKeyMissingProp(AttributeReadContext, String),
    #[error("attribute function for prop {0} has a key {1} but prop kind is {2} not a map)")]
    AttributeFuncForKeySetOnWrongKind(PropId, String, PropKind),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error(transparent)]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("Missing ExternalProvider {1} for AttributePrototypeArgument {1}")]
    AttributePrototypeArgumentMissingExternalProvider(
        AttributePrototypeArgumentId,
        ExternalProviderId,
    ),
    #[error("AttributePrototypeArgument {0} missing FuncArgument {1}")]
    AttributePrototypeArgumentMissingFuncArgument(AttributePrototypeArgumentId, FuncArgumentId),
    #[error("Missing InternalProvider {1} for AttributePrototypeArgument {1}")]
    AttributePrototypeArgumentMissingInternalProvider(
        AttributePrototypeArgumentId,
        InternalProviderId,
    ),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error("map item prop {0} has both custom key prototypes and custom prop only prototype")]
    ConflictingMapKeyPrototypes(PropId),
    #[error("Cannot find Socket for explicit InternalProvider {0}")]
    ExplicitInternalProviderMissingSocket(InternalProviderId),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("Cannot find Socket for ExternalProvider {0}")]
    ExternalProviderMissingSocket(ExternalProviderId),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error("Installed func id {0} does not exist")]
    InstalledFuncMissing(FuncId),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("Installed schema id {0} does not exist")]
    InstalledSchemaMissing(SchemaId),
    #[error("Installed schema variant {0} does not exist")]
    InstalledSchemaVariantMissing(SchemaVariantId),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("Missing Prop {1} for InternalProvider {1}")]
    InternalProviderMissingProp(InternalProviderId, PropId),
    #[error("Cannot package func with backend kind of {0}")]
    InvalidFuncBackendKind(FuncBackendKind),
    #[error("Cannot package func with backend response type of {0}")]
    InvalidFuncBackendResponseType(FuncBackendResponseType),
    #[error("Leaf Function {0} has invalid argument {1}")]
    InvalidLeafArgument(FuncId, String),
    #[error("Missing AttributePrototype {0} for explicit InternalProvider {1}")]
    MissingAttributePrototypeForInputSocket(AttributePrototypeId, InternalProviderId),
    #[error("Missing AttributePrototype {0} for ExternalProvider {1}")]
    MissingAttributePrototypeForOutputSocket(AttributePrototypeId, ExternalProviderId),
    #[error("Missing Func {1} for AttributePrototype {0}")]
    MissingAttributePrototypeFunc(AttributePrototypeId, FuncId),
    #[error("Func {0} missing from exported funcs")]
    MissingExportedFunc(FuncId),
    #[error("Cannot find FuncArgument {0} for Func {1}")]
    MissingFuncArgument(String, FuncId),
    #[error("Package asked for a function with the unique id {0} but none could be found")]
    MissingFuncUniqueId(String),
    #[error("Cannot find InternalProvider for Prop {0}")]
    MissingInternalProviderForProp(PropId),
    #[error("Cannot find InternalProvider for Socket named {0}")]
    MissingInternalProviderForSocketName(String),
    #[error("Intrinsic function {0} not found")]
    MissingIntrinsicFunc(String),
    #[error("Intrinsic function (0) argument {1} not found")]
    MissingIntrinsicFuncArgument(String, String),
    #[error("Cannot find item prop for installed map prop {0}")]
    MissingItemPropForMapProp(PropId),
    #[error("Cannot find installed prop {0}")]
    MissingProp(PropId),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    PkgSpec(#[from] SpecError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error("prop spec structure is invalid: {0}")]
    PropSpecChildrenInvalid(String),
    #[error(transparent)]
    PropTree(#[from] PropTreeError),
    #[error("prop tree structure is invalid: {0}")]
    PropTreeInvalid(String),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Socket(#[from] SocketError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("standard model relationship {0} missing belongs_to for {1} with id {2}")]
    StandardModelMissingBelongsTo(&'static str, &'static str, String),
    #[error("standard model relationship {0} found multiple belongs_to for {1} with id {2}")]
    StandardModelMultipleBelongsTo(&'static str, &'static str, String),
    #[error(transparent)]
    UrlParse(#[from] ParseError),
    #[error("Validation creation error: {0}")]
    Validation(#[from] ValidationPrototypeError),
}

impl PkgError {
    fn prop_tree_invalid(message: impl Into<String>) -> Self {
        Self::PropTreeInvalid(message.into())
    }

    fn prop_spec_children_invalid(message: impl Into<String>) -> Self {
        Self::PropSpecChildrenInvalid(message.into())
    }
}

pub type PkgResult<T> = Result<T, PkgError>;

impl TryFrom<FuncBackendKind> for FuncSpecBackendKind {
    type Error = PkgError;

    fn try_from(value: FuncBackendKind) -> Result<Self, Self::Error> {
        Ok(match value {
            FuncBackendKind::JsAttribute => FuncSpecBackendKind::JsAttribute,
            FuncBackendKind::JsReconciliation => FuncSpecBackendKind::JsReconciliation,
            FuncBackendKind::JsAction => FuncSpecBackendKind::JsAction,
            FuncBackendKind::JsValidation => FuncSpecBackendKind::JsValidation,
            _ => return Err(PkgError::InvalidFuncBackendKind(value)),
        })
    }
}

impl From<FuncSpecBackendKind> for FuncBackendKind {
    fn from(value: FuncSpecBackendKind) -> Self {
        match value {
            FuncSpecBackendKind::JsAttribute => FuncBackendKind::JsAttribute,
            FuncSpecBackendKind::JsReconciliation => FuncBackendKind::JsReconciliation,
            FuncSpecBackendKind::JsAction => FuncBackendKind::JsAction,
            FuncSpecBackendKind::JsValidation => FuncBackendKind::JsValidation,
        }
    }
}

impl TryFrom<FuncBackendResponseType> for FuncSpecBackendResponseType {
    type Error = PkgError;

    fn try_from(value: FuncBackendResponseType) -> Result<Self, Self::Error> {
        Ok(match value {
            FuncBackendResponseType::Action => FuncSpecBackendResponseType::Action,
            FuncBackendResponseType::Array => FuncSpecBackendResponseType::Array,
            FuncBackendResponseType::Boolean => FuncSpecBackendResponseType::Boolean,
            FuncBackendResponseType::CodeGeneration => FuncSpecBackendResponseType::CodeGeneration,
            FuncBackendResponseType::Confirmation => FuncSpecBackendResponseType::Confirmation,
            FuncBackendResponseType::Integer => FuncSpecBackendResponseType::Integer,
            FuncBackendResponseType::Json => FuncSpecBackendResponseType::Json,
            FuncBackendResponseType::Map => FuncSpecBackendResponseType::Map,
            FuncBackendResponseType::Object => FuncSpecBackendResponseType::Object,
            FuncBackendResponseType::Qualification => FuncSpecBackendResponseType::Qualification,
            FuncBackendResponseType::String => FuncSpecBackendResponseType::String,
            FuncBackendResponseType::Validation => FuncSpecBackendResponseType::Validation,

            _ => return Err(PkgError::InvalidFuncBackendResponseType(value)),
        })
    }
}

impl From<FuncSpecBackendResponseType> for FuncBackendResponseType {
    fn from(value: FuncSpecBackendResponseType) -> Self {
        match value {
            FuncSpecBackendResponseType::Action => FuncBackendResponseType::Action,
            FuncSpecBackendResponseType::Array => FuncBackendResponseType::Array,
            FuncSpecBackendResponseType::Boolean => FuncBackendResponseType::Boolean,
            FuncSpecBackendResponseType::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            FuncSpecBackendResponseType::Reconciliation => FuncBackendResponseType::Reconciliation,
            FuncSpecBackendResponseType::Confirmation => FuncBackendResponseType::Confirmation,
            FuncSpecBackendResponseType::Integer => FuncBackendResponseType::Integer,
            FuncSpecBackendResponseType::Json => FuncBackendResponseType::Json,
            FuncSpecBackendResponseType::Map => FuncBackendResponseType::Map,
            FuncSpecBackendResponseType::Object => FuncBackendResponseType::Object,
            FuncSpecBackendResponseType::Qualification => FuncBackendResponseType::Qualification,
            FuncSpecBackendResponseType::String => FuncBackendResponseType::String,
            FuncSpecBackendResponseType::Validation => FuncBackendResponseType::Validation,
        }
    }
}
