use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::attribute::value::view::PropertiesWithoutCode;
use crate::{
    component::ComponentKind, func::binding_return_value::FuncBindingReturnValueId,
    AttributeReadContext, AttributeValue, AttributeValueError, Component, ComponentId, DalContext,
    EncryptedSecret, ExternalProviderId, FuncBindingReturnValue, InternalProvider,
    InternalProviderError, Prop, PropError, PropId, SchemaVariantId, SecretError, SecretId,
    StandardModel, StandardModelError, System,
};

type ComponentViewResult<T> = Result<T, ComponentViewError>;

#[derive(Error, Debug)]
pub enum ComponentViewError {
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error(transparent)]
    Secret(#[from] SecretError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("json pointer not found: {1:?} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error("bad attribute read context {0}")]
    BadAttributeReadContext(String),
    #[error("component not found {0}")]
    NotFound(ComponentId),
    #[error("schema variant not found for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("no root prop found for schema variant {0}")]
    NoRootProp(SchemaVariantId),
    #[error("func binding return value not found {0}")]
    FuncBindingReturnValueNotFound(FuncBindingReturnValueId),
    #[error("no internal provider for prop {0}")]
    NoInternalProvider(PropId),
    #[error("no attribute value found for context {0:?}")]
    NoAttributeValue(AttributeReadContext),
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ComponentView {
    pub system: Option<System>,
    pub kind: ComponentKind,
    pub properties: Value,
}

impl Default for ComponentView {
    fn default() -> Self {
        Self {
            system: Default::default(),
            kind: Default::default(),
            properties: serde_json::json!({}),
        }
    }
}

impl ComponentView {
    pub async fn for_context(
        ctx: &DalContext,
        context: AttributeReadContext,
        include_code: bool,
    ) -> ComponentViewResult<ComponentView> {
        let component_id = match context.component_id() {
            Some(c) => c,
            None => {
                return Err(ComponentViewError::BadAttributeReadContext(
                    "component_id is required".to_string(),
                ));
            }
        };

        let component = Component::get_by_id(ctx, &component_id)
            .await?
            .ok_or(ComponentViewError::NotFound(component_id))?;

        // Perhaps get_by_id should just do this? -- Adam
        let system = match context.system_id() {
            Some(system_id) => System::get_by_id(ctx, &system_id).await?,
            None => None,
        };

        let schema_variant_id = context
            .schema_variant_id()
            .ok_or_else(|| ComponentViewError::NoSchemaVariant(*component.id()))?;

        let prop = Prop::find_root_for_schema_variant(ctx, schema_variant_id)
            .await?
            .ok_or(ComponentViewError::NoRootProp(schema_variant_id))?;

        let implicit_provider = InternalProvider::get_for_prop(ctx, *prop.id())
            .await?
            .ok_or_else(|| ComponentViewError::NoInternalProvider(*prop.id()))?;

        let value_context = AttributeReadContext {
            internal_provider_id: Some(*implicit_provider.id()),
            prop_id: Some(PropId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            ..context
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
            .unwrap_or(&serde_json::Value::Null);

        Ok(ComponentView {
            system,
            kind: *component.kind(),
            properties: match include_code {
                true => properties.clone(),
                false => {
                    PropertiesWithoutCode::drop_root_code_tree_if_applicable(properties.clone())?
                }
            },
        })
    }

    pub async fn reencrypt_secrets(
        ctx: &DalContext,
        component: &mut veritech_client::ComponentView,
    ) -> Result<(), ComponentViewError> {
        if component.kind != veritech_client::ComponentKind::Credential {
            return Ok(());
        }

        // If it's a credential it's already unencrypted
        if let Some(object) = component
            .properties
            .as_object_mut()
            .and_then(|obj| obj.get_mut("root").and_then(|obj| obj.as_object_mut()))
        {
            // Note: we can't know which fields are WidgetKind::SecretSelect as we lose information by being so low on the stack
            // So for now we will try to decrypt every integer root field, which kinda suck
            //
            // TODO: traverse tree and decrypt leafs
            for (_key, value) in object {
                if let Some(raw_id) = value.as_i64() {
                    let decrypted_secret = EncryptedSecret::get_by_id(ctx, &raw_id.into())
                        .await?
                        .ok_or_else(|| ComponentViewError::SecretNotFound(raw_id.into()))?
                        .decrypt(ctx)
                        .await?;
                    let encoded = ctx
                        .encryption_key()
                        .encrypt_and_encode(&serde_json::to_string(&decrypted_secret.message())?);

                    *value = serde_json::to_value(&decrypted_secret)?;
                    match value.pointer_mut("/message") {
                        Some(v) => {
                            *v = serde_json::json!({
                                "cycloneEncryptedDataMarker": true,
                                "encryptedSecret": encoded
                            })
                        }
                        None => {
                            return Err(ComponentViewError::JSONPointerNotFound(
                                value.clone(),
                                "/message".to_owned(),
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
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
            system: view.system.map(|system| veritech_client::SystemView {
                name: system.name().to_owned(),
            }),
            kind: view.kind.into(),
            properties: view.properties,
        }
    }
}
