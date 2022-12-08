//! This module contains logic for populating "/root/code" in a [`SchemaVariant`](crate::SchemaVariant).

use serde::Deserialize;
use serde::Serialize;

use crate::func::argument::FuncArgumentId;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, AttributeValueError,
    ComponentId, DalContext, Func, FuncError, FuncId, InternalProvider, InternalProviderError,
    Prop, PropId, PropKind, SchemaVariant, SchemaVariantId, StandardModel, WsEvent, WsPayload,
};

/// A leaf representing a single "code generation" for a [`SchemaVariant`](crate::SchemaVariant).
pub struct CodeGenerationLeaf {
    /// The starting point for leaf subtree.
    tree_prop_id: PropId,
    /// The string [`Prop`](crate::Prop) containing the generated code.
    code_prop_id: PropId,
    /// The string [`Prop`](crate::Prop) containing the generated code format.
    format_prop_id: PropId,
}

impl CodeGenerationLeaf {
    pub fn tree_prop_id(&self) -> PropId {
        self.tree_prop_id
    }

    pub fn code_prop_id(&self) -> PropId {
        self.code_prop_id
    }

    pub fn format_prop_id(&self) -> PropId {
        self.format_prop_id
    }
}

impl SchemaVariant {
    pub async fn add_code_generation(
        ctx: &DalContext,
        func_id: FuncId,
        func_argument_id: FuncArgumentId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<CodeGenerationLeaf> {
        if schema_variant_id.is_none() {
            return Err(SchemaVariantError::InvalidSchemaVariant);
        }

        // Collect the root prop for the schema variant as we will need it to setup new props
        // and intelligence.
        let root_prop = Self::root_prop(ctx, schema_variant_id).await?;

        // The new prop is named after the func name since func names must be unique for a given
        // tenancy and visibility. If that changes, then this may break.
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;
        let mut tree_prop = Prop::new(ctx, func.name(), PropKind::Object, None).await?;
        tree_prop.set_hidden(ctx, true).await?;
        tree_prop
            .set_parent_prop(ctx, root_prop.code_prop_id)
            .await?;
        let tree_prop_id = *tree_prop.id();

        // Now, create the two child props of the new prop. These represent the code generation
        // response fields.
        let mut child_code_prop = Prop::new(ctx, "code", PropKind::String, None).await?;
        child_code_prop.set_hidden(ctx, true).await?;
        child_code_prop.set_parent_prop(ctx, tree_prop_id).await?;
        let child_code_prop_id = *child_code_prop.id();

        let mut child_format_prop = Prop::new(ctx, "format", PropKind::String, None).await?;
        child_format_prop.set_hidden(ctx, true).await?;
        child_format_prop.set_parent_prop(ctx, tree_prop_id).await?;
        let child_format_prop_id = *child_format_prop.id();

        // Finalize the schema variant (which will likely be done again).
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(SchemaVariantError::NotFound(schema_variant_id))?;
        schema_variant.finalize(ctx).await?;

        let code_gen_obj_read_context = AttributeReadContext {
            prop_id: Some(tree_prop_id),
            ..AttributeReadContext::default()
        };
        let code_gen_obj_value = AttributeValue::find_for_context(ctx, code_gen_obj_read_context)
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(
                code_gen_obj_read_context,
            ))?;
        let mut code_gen_obj_prototype = code_gen_obj_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;

        code_gen_obj_prototype.set_func_id(ctx, func_id).await?;

        // code gen depends on the whole domain of the schema variant
        let domain_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, root_prop.domain_prop_id)
                .await?
                .ok_or(InternalProviderError::NotFoundForProp(
                    root_prop.domain_prop_id,
                ))?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *code_gen_obj_prototype.id(),
            func_argument_id,
            *domain_implicit_internal_provider.id(),
        )
        .await?;

        Ok(CodeGenerationLeaf {
            tree_prop_id,
            code_prop_id: child_code_prop_id,
            format_prop_id: child_format_prop_id,
        })
    }
}

// NOTE(nick): consider moving this somewhere else.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CodeGeneratedPayload {
    component_id: ComponentId,
}

// NOTE(nick): consider moving this somewhere else.
impl WsEvent {
    pub fn code_generated(ctx: &DalContext, component_id: ComponentId) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::CodeGenerated(CodeGeneratedPayload { component_id }),
        )
    }
}
