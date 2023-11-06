use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::{
    component::ComponentKind, func::binding_return_value::FuncBindingReturnValueId,
    AttributeReadContext, AttributeValue, AttributeValueError, Component, ComponentId, DalContext,
    FuncBindingReturnValue, InternalProvider, InternalProviderError, PropError, PropId,
    SchemaVariantId, SecretError, SecretId, StandardModel, StandardModelError,
};

pub mod debug;
pub mod properties;

pub use debug::{AttributeDebugView, ComponentDebugView};
pub use properties::ComponentViewProperties;

type ComponentViewResult<T> = Result<T, ComponentViewError>;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ComponentViewError {
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error("component error: {0}")]
    Component(String),
    #[error("func binding return value not found {0}")]
    FuncBindingReturnValueNotFound(FuncBindingReturnValueId),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("json pointer not found: {1:?} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error("no attribute value found for context {0:?}")]
    NoAttributeValue(AttributeReadContext),
    #[error("no internal provider for prop {0}")]
    NoInternalProvider(PropId),
    #[error("no root prop found for schema variant {0}")]
    NoRootProp(SchemaVariantId),
    #[error("schema variant not found for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("component not found {0}")]
    NotFound(ComponentId),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Secret(#[from] SecretError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComponentView {
    pub kind: ComponentKind,
    pub properties: Value,
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            kind: Default::default(),
            properties: serde_json::json!({}),
        }
    }
}

impl ComponentView {
    pub async fn new(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentViewResult<ComponentView> {
        let deleted_ctx = &ctx.clone_with_delete_visibility();
        let component = Component::get_by_id(deleted_ctx, &component_id)
            .await?
            .ok_or(ComponentViewError::NotFound(component_id))?;
        let schema_variant = component
            .schema_variant(ctx)
            .await
            .map_err(|e| ComponentViewError::Component(e.to_string()))?
            .ok_or_else(|| ComponentViewError::NoSchemaVariant(*component.id()))?;

        let root_prop_id = schema_variant
            .root_prop_id()
            .ok_or(ComponentViewError::NoRootProp(*schema_variant.id()))?;
        let implicit_provider = InternalProvider::find_for_prop(ctx, *root_prop_id)
            .await?
            .ok_or_else(|| ComponentViewError::NoInternalProvider(*root_prop_id))?;

        let value_context = AttributeReadContext {
            internal_provider_id: Some(*implicit_provider.id()),
            component_id: Some(component_id),
            ..AttributeReadContext::default()
        };

        let attribute_value = AttributeValue::find_for_context(ctx, value_context)
            .await?
            .ok_or(ComponentViewError::NoAttributeValue(value_context))?;

        let properties_func_binding_return_value =
            FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
                .await?
                .ok_or_else(|| {
                    ComponentViewError::FuncBindingReturnValueNotFound(
                        attribute_value.func_binding_return_value_id(),
                    )
                })?;
        let properties = properties_func_binding_return_value
            .value()
            .unwrap_or(&Value::Null);

        Ok(ComponentView {
            kind: *component.kind(),
            properties: properties.clone(),
        })
    }
}

impl From<ComponentKind> for veritech_client::ComponentKind {
    fn from(view: ComponentKind) -> Self {
        match view {
            ComponentKind::Standard => Self::Standard,
            ComponentKind::Credential => Self::Credential,
        }
    }
}

impl From<ComponentView> for veritech_client::ComponentView {
    fn from(view: ComponentView) -> Self {
        Self {
            // Filters internal data out, leaving only what is useful
            kind: view.kind.into(),
            properties: view.properties,
        }
    }
}
