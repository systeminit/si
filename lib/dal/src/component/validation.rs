//! This module contains the ability to interact with validations for
//! [`Component(s)`](crate::Component).

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::attribute::value::AttributeValue;
use crate::component::ComponentResult;
use crate::func::backend::{
    js_validation::FuncBackendJsValidationArgs, validation::FuncBackendValidationArgs,
};
use crate::func::binding::FuncBinding;
use crate::func::binding_return_value::FuncBindingReturnValue;
use crate::ComponentError;
use crate::{
    AttributeReadContext, Component, DalContext, ExternalProviderId, Func, FuncBackendKind,
    InternalProviderId, PropError, PropId, StandardModel, ValidationPrototype, ValidationResolver,
};

impl Component {
    pub async fn check_single_validation(
        &self,
        ctx: &DalContext,
        validation_prototype: &ValidationPrototype,
        value_cache: &mut HashMap<PropId, (Option<Value>, AttributeValue)>,
    ) -> ComponentResult<()> {
        let base_attribute_read_context = AttributeReadContext {
            prop_id: None,
            external_provider_id: Some(ExternalProviderId::NONE),
            internal_provider_id: Some(InternalProviderId::NONE),
            component_id: Some(self.id),
        };

        let prop_id = validation_prototype.context().prop_id();

        let (maybe_value, attribute_value) = match value_cache.get(&prop_id) {
            Some((value, attribute_value)) => (value.to_owned(), attribute_value.clone()),
            None => {
                let attribute_read_context = AttributeReadContext {
                    prop_id: Some(prop_id),
                    ..base_attribute_read_context
                };
                let attribute_value = AttributeValue::find_for_context(ctx, attribute_read_context)
                    .await?
                    .ok_or(ComponentError::AttributeValueNotFoundForContext(
                        attribute_read_context,
                    ))?;

                let value = match FuncBindingReturnValue::get_by_id(
                    ctx,
                    &attribute_value.func_binding_return_value_id(),
                )
                .await?
                {
                    Some(func_binding_return_value) => func_binding_return_value.value().cloned(),
                    None => None,
                };

                value_cache.insert(prop_id, (value.clone(), attribute_value.clone()));
                (value, attribute_value)
            }
        };

        let func = Func::get_by_id(ctx, &validation_prototype.func_id())
            .await?
            .ok_or_else(|| PropError::MissingFuncById(validation_prototype.func_id()))?;

        let mutated_args = match func.backend_kind() {
            FuncBackendKind::Validation => {
                // Deserialize the args, update the "value", and serialize the mutated args.
                let mut args = FuncBackendValidationArgs::deserialize(validation_prototype.args())?;
                args.validation = args.validation.update_value(&maybe_value)?;

                serde_json::to_value(args)?
            }
            FuncBackendKind::JsValidation => serde_json::to_value(FuncBackendJsValidationArgs {
                value: maybe_value.unwrap_or(serde_json::json!(null)),
            })?,
            kind => {
                return Err(ComponentError::InvalidFuncBackendKindForValidations(*kind));
            }
        };

        // TODO Load Before
        let before = vec![];

        // Now, we can load in the mutated args!
        let (func_binding, _) =
            FuncBinding::create_and_execute(ctx, mutated_args, *func.id(), before).await?;

        let attribute_value_id = *attribute_value.id();

        // Does a resolver already exist for this validation func and attribute value? If so, we
        // need to make sure the attribute_value_func_binding_return_value_id matches the
        // func_binding_return_value_id of the current attribute value, since it could be different
        // *even if the value is the same*. We also need to be sure to create a resolver for each
        // attribute_value_id, since the way func_bindings are cached means the validation func
        // won't be created for the same validation func + value, despite running this on a
        // completely different attribute value (or even prop).
        match ValidationResolver::find_for_attribute_value_and_validation_func(
            ctx,
            attribute_value_id,
            *func.id(),
        )
        .await?
        .pop()
        {
            Some(mut existing_resolver) => {
                existing_resolver
                    .set_validation_func_binding_id(ctx, func_binding.id())
                    .await?;
                existing_resolver
                    .set_attribute_value_func_binding_return_value_id(
                        ctx,
                        attribute_value.func_binding_return_value_id(),
                    )
                    .await?;
            }
            None => {
                ValidationResolver::new(
                    ctx,
                    *validation_prototype.id(),
                    attribute_value_id,
                    *func_binding.id(),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Check validations for [`Self`].
    pub async fn check_validations(&self, ctx: &DalContext) -> ComponentResult<()> {
        let schema_variant = self
            .schema_variant(ctx)
            .await?
            .ok_or(ComponentError::NoSchemaVariant(self.id))?;

        let validation_prototypes =
            ValidationPrototype::list_for_schema_variant(ctx, *schema_variant.id()).await?;

        // Cache data necessary for assembling func arguments. We do this since a prop can have
        // multiple validation prototypes within schema variant.
        let mut cache: HashMap<PropId, (Option<Value>, AttributeValue)> = HashMap::new();

        for validation_prototype in validation_prototypes {
            self.check_single_validation(ctx, &validation_prototype, &mut cache)
                .await?;
        }

        Ok(())
    }
}
