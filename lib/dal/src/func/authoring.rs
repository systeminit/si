//! This module contains backend functionality for the [`Func`] authoring experience.
//!
//! _Note_: all submodules are private since the [`FuncAuthoringClient`] is the primary interface.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use base64::Engine;
use base64::engine::general_purpose;
use si_events::FuncRunId;
use si_layer_cache::LayerDbError;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::action::prototype::{ActionKind, ActionPrototypeError};
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::value::AttributeValueError;
use crate::func::FuncKind;
use crate::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind};
use crate::prop::PropError;
use crate::schema::variant::authoring::{VariantAuthoringClient, VariantAuthoringError};
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::socket::output::OutputSocketError;
use crate::{
    AttributePrototype, AttributePrototypeId, ComponentError, ComponentId, DalContext, Func,
    FuncBackendKind, FuncBackendResponseType, FuncError, FuncId, SchemaVariant, SchemaVariantError,
    SchemaVariantId, TransactionsError, WorkspaceSnapshotError, WsEvent, WsEventError,
};

use super::binding::attribute::AttributeBinding;
use super::binding::{
    AttributeArgumentBinding, AttributeFuncDestination, EventualParent, FuncBinding,
    FuncBindingError,
};
use super::runner::{FuncRunner, FuncRunnerError};

mod create;
mod execute;
mod ts_types;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAuthoringError {
    #[error(
        "action with kind ({0}) already exists for schema variant ({1}), cannot have two non-manual actions for the same kind in the same schema variant"
    )]
    ActionKindAlreadyExists(ActionKind, SchemaVariantId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("cannot unlock non-default schema variant: {0}")]
    CannotUnlockNonDefaultSchemaVariant(SchemaVariantId),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func bindings error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func named \"{0}\" already exists in this change set")]
    FuncNameExists(String),
    #[error("Function options are incompatible with variant")]
    FuncOptionsAndVariantMismatch,
    #[error("func run value sender is gone without sending a value")]
    FuncRunGone,
    #[error("func run error: {0}")]
    FuncRunner(#[from] FuncRunnerError),
    #[error("func runner has failed to send a value and exited")]
    FuncRunnerSend,
    #[error("invalid func kind for creation: {0}")]
    InvalidFuncKindForCreation(FuncKind),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("no input location given for attribute prototype id ({0}) and func argument id ({1})")]
    NoInputLocationGiven(AttributePrototypeId, FuncArgumentId),
    #[error("no output location given for func: {0}")]
    NoOutputLocationGiven(FuncId),
    #[error("func ({0}) is not runnable with kind: {1}")]
    NotRunnable(FuncId, FuncKind),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("tokio task join error: {0}")]
    TokioTaskJoin(#[from] tokio::task::JoinError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("unexpected func kind ({0}) creating attribute func")]
    UnexpectedFuncKindCreatingAttributeFunc(FuncKind),
    #[error(
        "unexpected func kind ({0}) for func ({1}) when creating func argument (expected an attribute func kind)"
    )]
    UnexpectedFuncKindCreatingFuncArgument(FuncId, FuncKind),
    #[error("variant authoring error: {0}")]
    VariantAuthoringClient(#[from] Box<VariantAuthoringError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

type FuncAuthoringResult<T> = Result<T, FuncAuthoringError>;

/// This unit struct is the primary interface for the [`Func`](crate::Func) authoring experience.
#[derive(Debug)]
pub struct FuncAuthoringClient;

impl FuncAuthoringClient {
    /// Creates a new Attribute Func and returns it
    #[instrument(
        name = "func.authoring.create_new_attribute_func",
        level = "info",
        skip(ctx)
    )]
    pub async fn create_new_attribute_func(
        ctx: &DalContext,
        name: Option<String>,
        eventual_parent: Option<EventualParent>,
        output_location: AttributeFuncDestination,
        argument_bindings: Vec<AttributeArgumentBinding>,
    ) -> FuncAuthoringResult<Func> {
        if let Some(eventual_parent) = eventual_parent {
            eventual_parent.error_if_locked(ctx).await?;
        }
        let func = create::create_attribute_func(
            ctx,
            name,
            eventual_parent,
            output_location,
            argument_bindings,
        )
        .await?;

        Ok(func)
    }

    /// Creates a new Action Func and returns it
    #[instrument(
        name = "func.authoring.create_new_action_func",
        level = "info",
        skip(ctx)
    )]
    pub async fn create_new_action_func(
        ctx: &DalContext,
        name: Option<String>,
        action_kind: ActionKind,
        schema_variant_id: SchemaVariantId,
    ) -> FuncAuthoringResult<Func> {
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
        let func = create::create_action_func(ctx, name, action_kind, schema_variant_id).await?;
        Ok(func)
    }

    /// Creates a new Management Func and returns it
    #[instrument(
        name = "func.authoring.create_new_management_func",
        level = "info",
        skip(ctx)
    )]
    pub async fn create_new_management_func(
        ctx: &DalContext,
        name: Option<String>,
        schema_variant_id: SchemaVariantId,
    ) -> FuncAuthoringResult<Func> {
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
        let func = create::create_management_func(ctx, name, schema_variant_id).await?;
        Ok(func)
    }

    /// Creates a new Code Gen or Qualification Func and returns it
    #[instrument(
        name = "func.authoring.create_new_leaf_func",
        level = "info",
        skip(ctx)
    )]
    pub async fn create_new_leaf_func(
        ctx: &DalContext,
        name: Option<String>,
        leaf_kind: LeafKind,
        eventual_parent: EventualParent,
        inputs: &[LeafInputLocation],
    ) -> FuncAuthoringResult<Func> {
        eventual_parent.error_if_locked(ctx).await?;
        let func = create::create_leaf_func(ctx, name, leaf_kind, eventual_parent, inputs).await?;
        Ok(func)
    }

    /// Create a new Auth func and return it
    #[instrument(
        name = "func.authoring.create_new_auth_func",
        level = "info",
        skip(ctx)
    )]
    pub async fn create_new_auth_func(
        ctx: &DalContext,
        name: Option<String>,
        schema_variant_id: SchemaVariantId,
    ) -> FuncAuthoringResult<Func> {
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
        let func = create::create_authentication_func(ctx, name, schema_variant_id).await?;
        Ok(func)
    }

    /// Performs a "test" [`Func`] execution and returns the [`FuncRunId`](si_events::FuncRun).
    #[instrument(name = "func.authoring.test_execute_func", level = "info", skip(ctx))]
    pub async fn test_execute_func(
        ctx: &DalContext,
        id: FuncId,
        args: serde_json::Value,
        maybe_updated_code: Option<String>,
        component_id: ComponentId,
    ) -> FuncAuthoringResult<FuncRunId> {
        let mut func = Func::get_by_id(ctx, id).await?;

        // If updated code is provided, and it differs from the existing code, modify the function.
        if let Some(updated_code) = maybe_updated_code {
            let encoded_updated_code = Some(general_purpose::STANDARD_NO_PAD.encode(updated_code));
            if encoded_updated_code != func.code_base64 {
                func = Func::modify_by_id(ctx, id, |func| {
                    func.code_base64 = encoded_updated_code;
                    Ok(())
                })
                .await?;
            }
        }

        let is_intrinsic = func.is_intrinsic();
        let (func_run_id, result_channel) =
            FuncRunner::run_test(ctx, func, args, component_id).await?;

        let func_run_value = result_channel
            .await
            .map_err(|_| FuncAuthoringError::FuncRunnerSend)??;

        let content_value: Option<si_events::CasValue> =
            func_run_value.value().cloned().map(Into::into);
        let content_unprocessed_value: Option<si_events::CasValue> =
            func_run_value.unprocessed_value().cloned().map(Into::into);

        let value_address = match content_value {
            Some(value) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )?
                    .0,
            ),
            None => None,
        };

        let unprocessed_value_address = match content_unprocessed_value {
            Some(value) => Some(
                ctx.layer_db()
                    .cas()
                    .write(
                        Arc::new(value.into()),
                        None,
                        ctx.events_tenancy(),
                        ctx.events_actor(),
                    )?
                    .0,
            ),
            None => None,
        };

        if !is_intrinsic {
            FuncRunner::update_run(ctx, func_run_value.func_run_id(), |func_run| {
                func_run.set_success(unprocessed_value_address, value_address);
            })
            .await?;
        }

        Ok(func_run_id)
    }

    /// Executes a [`Func`].
    #[instrument(name = "func.authoring.execute_func", level = "info", skip(ctx))]
    pub async fn execute_func(ctx: &DalContext, id: FuncId) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id(ctx, id).await?;

        match func.kind {
            FuncKind::Qualification | FuncKind::CodeGeneration | FuncKind::Attribute => {
                execute::execute_attribute_func(ctx, &func).await?
            }
            FuncKind::Action => {
                // TODO(nick): fully restore or wait for actions v2. Essentially, we need to run
                // every prototype using the func id for every component.
                warn!("skipping action execution...");
                return Ok(());
            }
            kind => return Err(FuncAuthoringError::NotRunnable(id, kind)),
        };

        Ok(())
    }

    /// Creates a [`FuncArgument`].
    /// Returns an error if the [`Func`] is locked
    #[instrument(name = "func.authoring.create_func_argument", level = "info", skip_all)]
    pub async fn create_func_argument(
        ctx: &DalContext,
        id: FuncId,
        name: impl Into<String>,
        kind: FuncArgumentKind,
        element_kind: Option<FuncArgumentKind>,
    ) -> FuncAuthoringResult<FuncArgument> {
        let func = Func::get_by_id(ctx, id).await?;
        // don't create a func argument if the function is locked
        func.error_if_locked()?;
        if func.kind != FuncKind::Attribute {
            return Err(FuncAuthoringError::UnexpectedFuncKindCreatingFuncArgument(
                func.id, func.kind,
            ));
        }

        let func_argument = FuncArgument::new(ctx, name, kind, element_kind, id).await?;

        Ok(func_argument)
    }

    /// Deletes a [`FuncArgument`].
    #[instrument(name = "func.authoring.delete_func_argument", level = "info", skip_all)]
    pub async fn delete_func_argument(
        ctx: &DalContext,
        id: FuncArgumentId,
    ) -> FuncAuthoringResult<()> {
        // don't delete func argument if func is locked
        let func_id = FuncArgument::get_func_id_for_func_arg_id(ctx, id).await?;
        let func = Func::get_by_id(ctx, func_id).await?;
        func.error_if_locked()?;

        for attribute_prototype_argument_id in
            FuncArgument::list_attribute_prototype_argument_ids(ctx, id).await?
        {
            AttributePrototypeArgument::remove(ctx, attribute_prototype_argument_id).await?;
        }

        FuncArgument::remove(ctx, id).await?;

        Ok(())
    }

    /// Creates an unlocked copy of the [`FuncId`]
    /// If the attached [`SchemaVariant`]s are locked, we also create unlocked copies of the variant(s)
    /// If a single [`SchemaVariantId`] is provided, we only clone the bindings for that [`SchemaVariant`], otherwise
    /// we clone the bindings for all currently attached [`SchemaVariants`]
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.authoring.create_unlocked_func_copy"
    )]
    pub async fn create_unlocked_func_copy(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: Option<SchemaVariantId>,
    ) -> FuncAuthoringResult<Func> {
        match schema_variant_id {
            Some(schema_variant_id) => {
                Self::create_unlocked_func_copy_for_single_variant(ctx, func_id, schema_variant_id)
                    .await
            }
            None => Self::create_unlocked_func_copy_for_all_schema_variants(ctx, func_id).await,
        }
    }

    async fn create_unlocked_func_copy_for_single_variant(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
    ) -> FuncAuthoringResult<Func> {
        let old_func = Func::get_by_id(ctx, func_id).await?;

        let schema = SchemaVariant::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;
        // is the current schema varaint already unlocked? if so, proceed
        let current_schema_variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
        let new_func = if !current_schema_variant.is_locked() {
            //already on an unlocked variant, just create unlocked copy of the func and reattach
            // bindings for that schema variant
            let unlocked_latest =
                FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;

            // now, create the unlocked copy of the func
            let new_func = old_func.create_unlocked_func_copy(ctx).await?;

            for binding in unlocked_latest {
                // for the binding, remove it and create the equivalent for the new one
                if binding.get_schema_variant() == Some(schema_variant_id) {
                    binding.port_binding_to_new_func(ctx, new_func.id).await?;
                }
            }

            let variant = SchemaVariant::get_by_id(ctx, schema_variant_id).await?;
            WsEvent::schema_variant_updated(ctx, schema, variant)
                .await?
                .publish_on_commit(ctx)
                .await?;
            new_func
        } else if current_schema_variant.is_default(ctx).await? {
            let new_schema_variant =
                VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
                    .await
                    .map_err(Box::new)?;
            let new_schema_variant_id = new_schema_variant.id();

            let unlocked_latest =
                FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;

            // now, create the unlocked copy of the func
            let new_func = old_func.create_unlocked_func_copy(ctx).await?;

            for binding in unlocked_latest {
                // for the binding, remove it and create the equivalent for the new one
                if binding.get_schema_variant() == Some(new_schema_variant_id) {
                    binding.port_binding_to_new_func(ctx, new_func.id).await?;
                }
            }
            let new_variant = SchemaVariant::get_by_id(ctx, new_schema_variant_id).await?;
            WsEvent::schema_variant_created(ctx, schema, new_variant)
                .await?
                .publish_on_commit(ctx)
                .await?;

            new_func
        } else {
            return Err(FuncAuthoringError::CannotUnlockNonDefaultSchemaVariant(
                schema_variant_id,
            ));
        };

        Ok(new_func)
    }

    /// Find all the latest [`SchemaVariant`]s that have bindings for the given [`FuncId`]
    /// If any of them are currently locked, create unlocked copies of the variants
    /// Then, create an unlocked copy of the current Func, delete the binding for the now 'old'
    /// func and recreate the exact binding for the newly unlocked copy for the potentially new,
    /// unlocked schema variants (as well as all existing unlocked schema variants)
    async fn create_unlocked_func_copy_for_all_schema_variants(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncAuthoringResult<Func> {
        let old_func = Func::get_by_id(ctx, func_id).await?;
        let mut new_schema_variants = vec![];
        // Create unlocked versions of all schema variants that are locked, default and have a bindings to old func
        for (schema_variant_id, _) in
            FuncBinding::get_bindings_for_default_and_unlocked_schema_variants(ctx, old_func.id)
                .await?
        {
            // todo figure out ws event here
            // See if the Schema Variant is locked
            if SchemaVariant::is_locked_by_id(ctx, schema_variant_id).await? {
                // if it's locked, create an unlocked copy of it
                // creating the unlocked copy will keep the func in question bound at the same place (so we'll see another binding for the
                // func after creating a copy)
                let new_schema_variant =
                    VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
                        .await
                        .map_err(Box::new)?;
                new_schema_variants.push(new_schema_variant);
            }
        }

        // now get the bindings for the unlocked, schema variants for the current func. We need this for later.
        // this will include any newly unlocked variants if they exist
        let unlocked_latest =
            FuncBinding::get_bindings_for_unlocked_schema_variants(ctx, func_id).await?;
        // create the unlocked copy of the func
        let new_func = old_func.create_unlocked_func_copy(ctx).await?;

        // loop through the other bindings and port them to the new func
        for binding in unlocked_latest {
            // for the binding, remove it and create the equivalent for the new one
            binding.port_binding_to_new_func(ctx, new_func.id).await?;
        }
        // now fire the event with all the new schema variants that have updated bindings with the new func
        for schema_variant in new_schema_variants {
            WsEvent::schema_variant_created(
                ctx,
                schema_variant.schema(ctx).await?.id(),
                schema_variant,
            )
            .await?
            .publish_on_commit(ctx)
            .await?;
        }

        Ok(new_func)
    }

    #[instrument(level = "info", name = "func.authoring.save_code", skip(ctx))]
    /// Save only the code for the given [`FuncId`]
    /// Returns an error if the [`Func`] is currently locked
    pub async fn save_code(
        ctx: &DalContext,
        func_id: FuncId,
        code: String,
    ) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id(ctx, func_id).await?;
        func.error_if_locked()?;
        Func::modify_by_id(ctx, func.id, |func| {
            func.code_base64 = Some(general_purpose::STANDARD_NO_PAD.encode(code));

            Ok(())
        })
        .await?;

        // enqueue DVU when the func is saved if it's for an attribute/codegen/qualification
        let attribute_prototypes = AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
        for attribute_prototype_id in attribute_prototypes {
            AttributeBinding::enqueue_dvu_for_impacted_values(ctx, attribute_prototype_id).await?;
        }
        Ok(())
    }

    /// Save metadata about the [`FuncId`]
    /// Returns an error if the [`Func`] is currently locked
    #[instrument(level = "info", name = "func.authoring.update_func", skip(ctx))]
    pub async fn update_func(
        ctx: &DalContext,
        func_id: FuncId,
        display_name: Option<String>,
        description: Option<String>,
    ) -> FuncAuthoringResult<Func> {
        let func = Func::get_by_id(ctx, func_id).await?;
        func.error_if_locked()?;
        let updated_func = Func::modify_by_id(ctx, func.id, |func| {
            display_name.clone_into(&mut func.display_name);
            description.clone_into(&mut func.description);
            Ok(())
        })
        .await?;
        Ok(updated_func)
    }

    /// Compiles types corresponding to "lang-js".
    pub fn compile_langjs_types() -> &'static str {
        ts_types::compile_langjs_types()
    }

    /// Compiles return types based on a [`FuncBackendResponseType`] and [`FuncBackendKind`].
    pub fn compile_return_types(
        response_type: FuncBackendResponseType,
        kind: FuncBackendKind,
    ) -> &'static str {
        ts_types::compile_return_types(response_type, kind)
    }

    /// Compiles return types based on the [`FuncBinding`] for the Func
    pub async fn compile_types_from_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncAuthoringResult<String> {
        Ok(FuncBinding::compile_types(ctx, func_id).await?)
    }
}
