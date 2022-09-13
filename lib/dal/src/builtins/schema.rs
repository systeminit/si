use crate::{
    component::ComponentKind,
    func::{
        backend::js_attribute::FuncBackendJsAttributeArgs,
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    AttributeReadContext, AttributeValue, AttributeValueError, BuiltinsResult, DalContext, Func,
    FuncBackendKind, FuncBackendResponseType, FuncError, FuncId, Prop, PropError, PropId, PropKind,
    Schema, SchemaKind, StandardModel,
};

mod docker;
mod kubernetes;
mod systeminit;

pub async fn migrate(ctx: &DalContext<'_, '_, '_>) -> BuiltinsResult<()> {
    systeminit::migrate(ctx).await?;
    docker::migrate(ctx).await?;
    kubernetes::migrate(ctx).await?;
    Ok(())
}

/// This unit struct (zero bytes) provides a singular place to index helpers for creating builtin
/// [`Schemas`](crate::Schema).
pub struct BuiltinSchemaHelpers;

impl BuiltinSchemaHelpers {
    pub async fn create_schema(
        ctx: &DalContext<'_, '_, '_>,
        schema_name: &str,
        schema_kind: &SchemaKind,
    ) -> BuiltinsResult<Option<Schema>> {
        // TODO(nick): there's one issue here. If the schema kind has changed, then this check will be
        // inaccurate. As a result, we will be unable to re-create the schema without manual intervention.
        // This should be fine since this code should likely only last as long as default schemas need to
        // be created... which is hopefully not long.... hopefully...
        let default_schema_exists = !Schema::find_by_attr(ctx, "name", &schema_name.to_string())
            .await?
            .is_empty();

        // TODO(nick): this should probably return an "AlreadyExists" error instead of "None", but
        // since the calling function would have to deal with the result similarly, this should suffice
        // for now.
        match default_schema_exists {
            true => Ok(None),
            false => {
                let schema =
                    Schema::new(ctx, schema_name, schema_kind, &ComponentKind::Standard).await?;
                Ok(Some(schema))
            }
        }
    }

    /// Creates a [`Prop`]. While a base [`AttributeReadContext`] is required for this function, it is
    /// only used when a parent [`PropId`] is provided.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_prop(
        ctx: &DalContext<'_, '_, '_>,
        prop_name: &str,
        prop_kind: PropKind,
        parent_prop_id: Option<PropId>,
    ) -> BuiltinsResult<Prop> {
        let prop = Prop::new(ctx, prop_name, prop_kind).await?;
        if let Some(parent_prop_id) = parent_prop_id {
            prop.set_parent_prop(ctx, parent_prop_id).await?;
        }
        Ok(prop)
    }

    pub async fn create_string_prop_with_default(
        ctx: &DalContext<'_, '_, '_>,
        prop_name: &str,
        default_string: String,
        parent_prop_id: Option<PropId>,
        _base_attribute_read_context: AttributeReadContext,
    ) -> BuiltinsResult<Prop> {
        let prop = Self::create_prop(ctx, prop_name, PropKind::String, parent_prop_id).await?;

        let mut func = Func::new(
            ctx,
            &format!("si:setDefaultToProp{:?}", prop.id()),
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::String,
        )
        .await
        .expect("cannot create func");
        func.set_handler(ctx, Some("defaultValue")).await?;
        func.set_code_base64(
            ctx,
            Some(base64::encode(&format!(
                "function defaultValue(component) {{ return \"{default_string}\"; }}"
            ))),
        )
        .await?;

        let (func_binding, func_binding_return_value) = FuncBinding::find_or_create_and_execute(
            ctx,
            // The default run doesn't have useful information, but it's just a reference for future reruns
            serde_json::to_value(FuncBackendJsAttributeArgs {
                component: veritech::ResolverFunctionComponent {
                    data: veritech::ComponentView {
                        properties: serde_json::json!({}),
                        system: None,
                        kind: veritech::ComponentKind::Standard,
                        resources: vec![],
                    },
                    parents: vec![],
                },
            })?,
            *func.id(),
        )
        .await?;

        let attribute_value_context = AttributeReadContext {
            prop_id: Some(*prop.id()),
            ..AttributeReadContext::default()
        };

        Prop::create_default_prototypes_and_values(ctx, *prop.id()).await?;

        let mut attribute_value = AttributeValue::find_for_context(ctx, attribute_value_context)
            .await?
            .ok_or(AttributeValueError::Missing)?;
        attribute_value
            .set_func_binding_id(ctx, *func_binding.id())
            .await?;
        attribute_value
            .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
            .await?;

        let mut attribute_prototype = attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        attribute_prototype.set_func_id(ctx, *func.id()).await?;

        Ok(prop)
    }

    /// Get the "si:identity" [`Func`](crate::Func) and execute (if necessary).
    pub async fn setup_identity_func(
        ctx: &DalContext<'_, '_, '_>,
    ) -> BuiltinsResult<(FuncId, FuncBindingId, FuncBindingReturnValueId)> {
        let identity_func_name = "si:identity".to_string();
        let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
            .await?
            .pop()
            .ok_or(FuncError::NotFoundByName(identity_func_name))?;
        let (identity_func_binding, identity_func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(
                ctx,
                serde_json::json![{ "identity": null }],
                *identity_func.id(),
            )
            .await?;
        Ok((
            *identity_func.id(),
            *identity_func_binding.id(),
            *identity_func_binding_return_value.id(),
        ))
    }

    /// Find the child of a [`Prop`](crate::Prop) by name.
    ///
    /// _Use with caution!_
    pub async fn find_child_prop_by_name(
        ctx: &DalContext<'_, '_, '_>,
        prop_id: PropId,
        child_prop_name: &str,
    ) -> BuiltinsResult<Prop> {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or_else(|| PropError::NotFound(prop_id, *ctx.visibility()))?;
        for current in prop.child_props(ctx).await? {
            if current.name() == child_prop_name {
                return Ok(current);
            }
        }
        Err(PropError::ExpectedChildNotFound(child_prop_name.to_string()).into())
    }
}
