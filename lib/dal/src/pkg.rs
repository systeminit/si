use std::convert::TryFrom;
use thiserror::Error;
use url::ParseError;

mod export;
mod import;

pub use export::export_pkg;
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
    AttributeValueError, ExternalProviderError, ExternalProviderId, FuncBackendKind,
    FuncBackendResponseType, FuncError, FuncId, InternalProviderError, InternalProviderId,
    PropError, PropId, SchemaError, SchemaId, SchemaVariantError, SchemaVariantId,
    StandardModelError, ValidationPrototypeError, WorkflowPrototypeError,
};

#[derive(Debug, Error)]
pub enum PkgError {
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error(transparent)]
    PkgSpec(#[from] SpecError),
    #[error("prop spec structure is invalid: {0}")]
    PropSpecChildrenInvalid(String),
    #[error(transparent)]
    PropTree(#[from] PropTreeError),
    #[error("prop tree structure is invalid: {0}")]
    PropTreeInvalid(String),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("Installed schema id {0} does not exist")]
    InstalledSchemaMissing(SchemaId),
    #[error("Installed func id {0} does not exist")]
    InstalledFuncMissing(FuncId),
    #[error("Installed schema variant {0} does not exist")]
    InstalledSchemaVariantMissing(SchemaVariantId),
    #[error("standard model relationship {0} missing belongs_to for {1} with id {2}")]
    StandardModelMissingBelongsTo(&'static str, &'static str, String),
    #[error("standard model relationship {0} found multiple belongs_to for {1} with id {2}")]
    StandardModelMultipleBelongsTo(&'static str, &'static str, String),
    #[error(transparent)]
    UrlParse(#[from] ParseError),
    #[error("Cannot package func with backend kind of {0}")]
    InvalidFuncBackendKind(FuncBackendKind),
    #[error("Cannot package func with backend response type of {0}")]
    InvalidFuncBackendResponseType(FuncBackendResponseType),
    #[error("Package asked for a function with the unique id {0} but none could be found")]
    MissingFuncUniqueId(String),
    #[error("Func {0} missing from exported funcs")]
    MissingExportedFunc(FuncId),
    #[error("Leaf Function {0} has invalid argument {1}")]
    InvalidLeafArgument(FuncId, String),
    #[error("Validation creation error: {0}")]
    Validation(#[from] ValidationPrototypeError),
    #[error("Workflow creation error: {0}")]
    Workflow(#[from] WorkflowPrototypeError),
    #[error("Action creation error: {0}")]
    Action(#[from] ActionPrototypeError),
    #[error("Missing AttributePrototype {0} for explicit InternalProvider {1}")]
    MissingAttributePrototypeForInputSocket(AttributePrototypeId, InternalProviderId),
    #[error("Missing AttributePrototype {0} for ExternalProvider {1}")]
    MissingAttributePrototypeForOutputSocket(AttributePrototypeId, ExternalProviderId),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error(transparent)]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error(transparent)]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    Socket(#[from] SocketError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error("Intrinsic function {0} not found")]
    MissingIntrinsicFunc(String),
    #[error("Intrinsic function (0) argument {1} not found")]
    MissingIntrinsicFuncArgument(String, String),
    #[error("Missing Func {1} for AttributePrototype {0}")]
    MissingAttributePrototypeFunc(AttributePrototypeId, FuncId),
    #[error("AttributePrototypeArgument {0} missing FuncArgument {1}")]
    AttributePrototypeArgumentMissingFuncArgument(AttributePrototypeArgumentId, FuncArgumentId),
    #[error("Missing InternalProvider {1} for AttributePrototypeArgument {1}")]
    AttributePrototypeArgumentMissingInternalProvider(
        AttributePrototypeArgumentId,
        InternalProviderId,
    ),
    #[error("Missing ExternalProvider {1} for AttributePrototypeArgument {1}")]
    AttributePrototypeArgumentMissingExternalProvider(
        AttributePrototypeArgumentId,
        ExternalProviderId,
    ),
    #[error("Missing Prop {1} for InternalProvider {1}")]
    InternalProviderMissingProp(InternalProviderId, PropId),
    #[error("Cannot find Socket for explicit InternalProvider {0}")]
    ExplicitInternalProviderMissingSocket(InternalProviderId),
    #[error("Cannot find Socket for ExternalProvider {0}")]
    ExternalProviderMissingSocket(ExternalProviderId),
    #[error("Cannot find FuncArgument {0} for Func {1}")]
    MissingFuncArgument(String, FuncId),
    #[error("Cannot find InternalProvider for Prop {0}")]
    MissingInternalProviderForProp(PropId),
    #[error("Cannot find InternalProvider for Socket named {0}")]
    MissingInternalProviderForSocketName(String),
    #[error("Cannot find installed prop {0}")]
    MissingProp(PropId),
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
            FuncBackendKind::JsCommand => FuncSpecBackendKind::JsCommand,
            FuncBackendKind::Json => FuncSpecBackendKind::Json,
            FuncBackendKind::JsValidation => FuncSpecBackendKind::JsValidation,
            FuncBackendKind::JsWorkflow => FuncSpecBackendKind::JsWorkflow,
            _ => return Err(PkgError::InvalidFuncBackendKind(value)),
        })
    }
}

impl From<FuncSpecBackendKind> for FuncBackendKind {
    fn from(value: FuncSpecBackendKind) -> Self {
        match value {
            FuncSpecBackendKind::JsAttribute => FuncBackendKind::JsAttribute,
            FuncSpecBackendKind::JsCommand => FuncBackendKind::JsCommand,
            FuncSpecBackendKind::Json => FuncBackendKind::Json,
            FuncSpecBackendKind::JsValidation => FuncBackendKind::JsValidation,
            FuncSpecBackendKind::JsWorkflow => FuncBackendKind::JsWorkflow,
        }
    }
}

impl TryFrom<FuncBackendResponseType> for FuncSpecBackendResponseType {
    type Error = PkgError;

    fn try_from(value: FuncBackendResponseType) -> Result<Self, Self::Error> {
        Ok(match value {
            FuncBackendResponseType::Array => FuncSpecBackendResponseType::Array,
            FuncBackendResponseType::Boolean => FuncSpecBackendResponseType::Boolean,
            FuncBackendResponseType::CodeGeneration => FuncSpecBackendResponseType::CodeGeneration,
            FuncBackendResponseType::Command => FuncSpecBackendResponseType::Command,
            FuncBackendResponseType::Confirmation => FuncSpecBackendResponseType::Confirmation,
            FuncBackendResponseType::Integer => FuncSpecBackendResponseType::Integer,
            FuncBackendResponseType::Json => FuncSpecBackendResponseType::Json,
            FuncBackendResponseType::Map => FuncSpecBackendResponseType::Map,
            FuncBackendResponseType::Object => FuncSpecBackendResponseType::Object,
            FuncBackendResponseType::Qualification => FuncSpecBackendResponseType::Qualification,
            FuncBackendResponseType::String => FuncSpecBackendResponseType::String,
            FuncBackendResponseType::Validation => FuncSpecBackendResponseType::Validation,
            FuncBackendResponseType::Workflow => FuncSpecBackendResponseType::Workflow,

            _ => return Err(PkgError::InvalidFuncBackendResponseType(value)),
        })
    }
}

impl From<FuncSpecBackendResponseType> for FuncBackendResponseType {
    fn from(value: FuncSpecBackendResponseType) -> Self {
        match value {
            FuncSpecBackendResponseType::Array => FuncBackendResponseType::Array,
            FuncSpecBackendResponseType::Boolean => FuncBackendResponseType::Boolean,
            FuncSpecBackendResponseType::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            FuncSpecBackendResponseType::Command => FuncBackendResponseType::Command,
            FuncSpecBackendResponseType::Confirmation => FuncBackendResponseType::Confirmation,
            FuncSpecBackendResponseType::Integer => FuncBackendResponseType::Integer,
            FuncSpecBackendResponseType::Json => FuncBackendResponseType::Json,
            FuncSpecBackendResponseType::Map => FuncBackendResponseType::Map,
            FuncSpecBackendResponseType::Object => FuncBackendResponseType::Object,
            FuncSpecBackendResponseType::Qualification => FuncBackendResponseType::Qualification,
            FuncSpecBackendResponseType::String => FuncBackendResponseType::String,
            FuncSpecBackendResponseType::Validation => FuncBackendResponseType::Validation,
            FuncSpecBackendResponseType::Workflow => FuncBackendResponseType::Workflow,
        }
    }
}
