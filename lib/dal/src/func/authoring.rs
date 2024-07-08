//! This module contains backend functionality for the [`Func`] authoring experience.
//!
//! How does the authoring loop work? While metadata fetching and mutation is self-explanatory, the
//! [`FuncAssociations`] subsystem is less so. Essentially, [`FuncAssociations`] are a "bag" that
//! come alongside [`FuncView::assemble`](crate::func::view::FuncView::assemble) and are mutated via
//! [`FuncAuthoringClient::save_func`].
//!
//! The existence, difference or absence of an entity or field within a [`FuncAssociations`] bag
//! dictates what we must do during mutation. New argument? Create one and send it back up. Removed
//! argument? Delete it and send it back up. Mutated argument? Change it and send it back up. The
//! payload is the same on both sides.
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

use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use si_events::FuncRunId;
use si_layer_cache::LayerDbError;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::action::prototype::{ActionKind, ActionPrototype, ActionPrototypeError};
use crate::attribute::prototype::argument::{
    AttributePrototypeArgument, AttributePrototypeArgumentError,
};
use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::value::AttributeValueError;
use crate::func::argument::{FuncArgument, FuncArgumentError, FuncArgumentId, FuncArgumentKind};
use crate::func::associations::{FuncAssociations, FuncAssociationsError};
use crate::func::view::FuncViewError;
use crate::func::FuncKind;
use crate::prop::PropError;
use crate::schema::variant::authoring::{VariantAuthoringClient, VariantAuthoringError};
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::socket::output::OutputSocketError;
use crate::{
    AttributePrototype, AttributePrototypeId, ComponentError, ComponentId, DalContext, Func,
    FuncBackendKind, FuncBackendResponseType, FuncError, FuncId, OutputSocketId, PropId,
    SchemaVariant, SchemaVariantError, SchemaVariantId, TransactionsError, WorkspaceSnapshotError,
    WsEvent, WsEventError,
};

use super::binding::action::ActionBinding;
use super::binding::attribute::AttributeBinding;
use super::binding::authentication::AuthBinding;
use super::binding::leaf::LeafBinding;
use super::binding::{
    AttributeArgumentBinding, AttributeFuncDestination, EventualParent, FuncBinding,
    FuncBindingError,
};
use super::runner::{FuncRunner, FuncRunnerError};
use super::{AttributePrototypeArgumentBag, AttributePrototypeBag};

mod create;
mod execute;
mod save;
mod ts_types;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum FuncAuthoringError {
    #[error("action with kind ({0}) already exists for schema variant ({1}), cannot have two non-manual actions for the same kind in the same schema variant")]
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
    #[error("func associations error: {0}")]
    FuncAssociations(#[from] FuncAssociationsError),
    #[error("func bindings error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func ({0}) with kind ({1}) cannot have associations: {2:?}")]
    FuncCannotHaveAssociations(FuncId, FuncKind, FuncAssociations),
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
    #[error("func view error: {0}")]
    FuncView(#[from] FuncViewError),
    #[error("invalid func associations ({0:?}) for func ({1}) of kind: {2}")]
    InvalidFuncAssociationsForFunc(FuncAssociations, FuncId, FuncKind),
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
    #[error("unexpected func kind ({0}) for func ({1}) when creating func argument (expected an attribute func kind)")]
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
    /// Creates a [`Func`] and returns the [result](CreatedFunc).
    #[instrument(name = "func.authoring.create_func", level = "info", skip(ctx))]
    pub async fn create_func(
        ctx: &DalContext,
        kind: FuncKind,
        name: Option<String>,
        options: Option<CreateFuncOptions>,
    ) -> FuncAuthoringResult<CreatedFunc> {
        let func = create::create(ctx, kind, name, options).await?;
        Ok(CreatedFunc {
            id: func.id,
            handler: func.handler.as_ref().map(|h| h.to_owned()),
            kind: func.kind,
            name: func.name.to_owned(),
            code: func.code_plaintext()?,
        })
    }
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
        let func = create::create(ctx, FuncKind::Attribute, name, None).await?;
        AttributeBinding::upsert_attribute_binding(
            ctx,
            func.id,
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
        let func = create::create(ctx, FuncKind::Action, name, None).await?;
        ActionBinding::create_action_binding(ctx, func.id, action_kind, schema_variant_id).await?;
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
        let func = match leaf_kind {
            LeafKind::CodeGeneration => {
                let func = create::create(ctx, FuncKind::CodeGeneration, name, None).await?;
                LeafBinding::create_leaf_func_binding(
                    ctx,
                    func.id,
                    eventual_parent,
                    leaf_kind,
                    inputs,
                )
                .await?;
                func
            }
            LeafKind::Qualification => {
                let func = create::create(ctx, FuncKind::Qualification, name, None).await?;
                LeafBinding::create_leaf_func_binding(
                    ctx,
                    func.id,
                    eventual_parent,
                    leaf_kind,
                    inputs,
                )
                .await?;
                func
            }
        };
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
        let func = create::create(ctx, FuncKind::Authentication, name, None).await?;
        AuthBinding::create_auth_binding(ctx, func.id, schema_variant_id).await?;
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
        let mut func = Func::get_by_id_or_error(ctx, id).await?;

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
                    )
                    .await?
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
                    )
                    .await?
                    .0,
            ),
            None => None,
        };

        ctx.layer_db()
            .func_run()
            .set_values_and_set_state_to_success(
                func_run_value.func_run_id(),
                unprocessed_value_address,
                value_address,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        Ok(func_run_id)
    }

    /// Executes a [`Func`].
    #[instrument(name = "func.authoring.execute_func", level = "info", skip(ctx))]
    pub async fn execute_func(ctx: &DalContext, id: FuncId) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id_or_error(ctx, id).await?;

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
    ) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id_or_error(ctx, id).await?;
        // don't create a func argument if the function is locked
        func.error_if_locked()?;
        if func.kind != FuncKind::Attribute {
            return Err(FuncAuthoringError::UnexpectedFuncKindCreatingFuncArgument(
                func.id, func.kind,
            ));
        }

        let _func_argument = FuncArgument::new(ctx, name, kind, element_kind, id).await?;

        Ok(())
    }

    /// Creates an [`AttributePrototype`]. Used when attaching an existing attribute
    /// function to a schema variant and/or component
    /// todo: remove once front end consumes new routes
    #[instrument(
        name = "func.authoring.create_attribute_prototype",
        level = "info",
        skip_all
    )]
    pub async fn create_attribute_prototype(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
        component_id: Option<ComponentId>,
        prop_id: Option<PropId>,
        output_socket_id: Option<OutputSocketId>,
        prototype_arguments: Vec<AttributePrototypeArgumentBag>,
    ) -> FuncAuthoringResult<AttributePrototypeId> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        if func.kind != FuncKind::Attribute {
            return Err(FuncAuthoringError::UnexpectedFuncKindCreatingFuncArgument(
                func.id, func.kind,
            ));
        }
        let prototype_bag = AttributePrototypeBag {
            id: AttributePrototypeId::NONE,
            component_id,
            schema_variant_id: Some(schema_variant_id),
            prop_id,
            output_socket_id,
            prototype_arguments,
        };
        let attribute_prototype_id =
            save::create_new_attribute_prototype(ctx, &prototype_bag, func_id, None).await?;
        save::save_attr_func_proto_arguments(
            ctx,
            attribute_prototype_id,
            prototype_bag.prototype_arguments.clone(),
        )
        .await?;
        Ok(attribute_prototype_id)
    }

    /// Updates an [`AttributePrototype`].
    /// todo: remove once front end consumes new routes
    #[instrument(
        name = "func.authoring.update_attribute_prototype",
        level = "info",
        skip_all
    )]
    pub async fn update_attribute_prototype(
        ctx: &DalContext,
        func_id: FuncId,
        attribute_prototype_id: AttributePrototypeId,
        prop_id: Option<PropId>,
        output_socket_id: Option<OutputSocketId>,
        prototype_arguments: Vec<AttributePrototypeArgumentBag>,
    ) -> FuncAuthoringResult<AttributePrototypeId> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        if func.kind != FuncKind::Attribute {
            return Err(FuncAuthoringError::UnexpectedFuncKindCreatingFuncArgument(
                func.id, func.kind,
            ));
        }
        let prototype_bag = AttributePrototypeBag::assemble(ctx, attribute_prototype_id).await?;

        let new_prototype_bag = AttributePrototypeBag {
            id: attribute_prototype_id,
            component_id: prototype_bag.component_id,
            schema_variant_id: prototype_bag.schema_variant_id,
            prop_id,
            output_socket_id,
            prototype_arguments,
        };
        let attribute_prototype_id =
            save::create_new_attribute_prototype(ctx, &new_prototype_bag, func_id, None).await?;
        save::save_attr_func_proto_arguments(
            ctx,
            attribute_prototype_id,
            new_prototype_bag.prototype_arguments.clone(),
        )
        .await?;
        Ok(attribute_prototype_id)
    }

    /// Removes an [`AttributePrototype`].
    /// todo: remove once front end consumes new routes
    #[instrument(
        name = "func.authoring.remove_attribute_prototype",
        level = "info",
        skip_all
    )]
    pub async fn remove_attribute_prototype(
        ctx: &DalContext,
        attribute_prototype_id: AttributePrototypeId,
    ) -> FuncAuthoringResult<()> {
        save::reset_attribute_prototype(ctx, attribute_prototype_id, true).await?;
        Ok(())
    }

    /// Deletes a [`FuncArgument`].
    #[instrument(name = "func.authoring.delete_func_argument", level = "info", skip_all)]
    pub async fn delete_func_argument(
        ctx: &DalContext,
        id: FuncArgumentId,
    ) -> FuncAuthoringResult<()> {
        // don't delete func argument if func is locked
        let func_id = FuncArgument::get_func_id_for_func_arg_id(ctx, id).await?;
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        func.error_if_locked()?;

        for attribute_prototype_argument_id in
            FuncArgument::list_attribute_prototype_argument_ids(ctx, id).await?
        {
            AttributePrototypeArgument::remove(ctx, attribute_prototype_argument_id).await?;
        }

        FuncArgument::remove(ctx, id).await?;

        Ok(())
    }

    /// Saves a [`Func`].
    /// todo: remove once front end consumes new routes
    #[instrument(name = "func.authoring.save_func", level = "info", skip(ctx))]
    pub async fn save_func(
        ctx: &DalContext,
        id: FuncId,
        display_name: Option<String>,
        name: String,
        description: Option<String>,
        code: Option<String>,
        associations: Option<FuncAssociations>,
    ) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id_or_error(ctx, id).await?;

        Func::modify_by_id(ctx, func.id, |func| {
            display_name.clone_into(&mut func.display_name);
            name.clone_into(&mut func.name);
            description.clone_into(&mut func.description);
            func.code_base64 = code
                .as_ref()
                .map(|code| general_purpose::STANDARD_NO_PAD.encode(code));

            Ok(())
        })
        .await?;

        if let Some(associations) = associations {
            let update_associations_start = tokio::time::Instant::now();
            save::update_associations(ctx, &func, associations).await?;
            debug!(%func.id, %func.kind,
                "updating associations took {:?}",
                update_associations_start.elapsed()
            );
        }

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
        let old_func = Func::get_by_id_or_error(ctx, func_id).await?;

        let schema = SchemaVariant::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;
        // is the current schema varaint already unlocked? if so, proceed
        let current_schema_variant =
            SchemaVariant::get_by_id_or_error(ctx, schema_variant_id).await?;
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
            // ws event

            new_func
        } else if current_schema_variant.is_default(ctx).await? {
            let new_schema_variant =
                VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
                    .await
                    .map_err(Box::new)?;
            let new_schema_variant_id = new_schema_variant.id();

            WsEvent::schema_variant_created(ctx, schema, new_schema_variant)
                .await?
                .publish_on_commit(ctx)
                .await?;
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

            new_func
        } else {
            return Err(FuncAuthoringError::CannotUnlockNonDefaultSchemaVariant(
                schema_variant_id,
            ));
        };

        let bindings: Vec<si_frontend_types::FuncBinding> = FuncBinding::for_func_id(ctx, func_id)
            .await?
            .into_iter()
            .map(|binding| binding.into())
            .collect();
        let new_bindings: Vec<si_frontend_types::FuncBinding> =
            FuncBinding::for_func_id(ctx, new_func.id)
                .await?
                .into_iter()
                .map(|binding| binding.into())
                .collect();
        WsEvent::func_bindings_updated(
            ctx,
            si_frontend_types::FuncBindings { bindings },
            Self::compile_types_from_bindings(ctx, func_id).await?,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        WsEvent::func_bindings_updated(
            ctx,
            si_frontend_types::FuncBindings {
                bindings: new_bindings,
            },
            Self::compile_types_from_bindings(ctx, new_func.id).await?,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

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
        let old_func = Func::get_by_id_or_error(ctx, func_id).await?;

        // Create unlocked versions of all schema variants that are locked, default and have a bindings to old func
        for (schema_variant_id, _) in
            FuncBinding::get_bindings_for_default_and_unlocked_schema_variants(ctx, old_func.id)
                .await?
        {
            // todo figure out ws event here
            // See if the Schema Variant is locked
            if SchemaVariant::is_locked_by_id(ctx, schema_variant_id).await? {
                let schema_id =
                    SchemaVariant::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;
                dbg!("creating unlocked copy");
                // if it's locked, create an unlocked copy of it
                // creating the unlocked copy will keep the func in question bound at the same place (so we'll see another binding for the
                // func after creating a copy)
                let new_schema_variant =
                    VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
                        .await
                        .map_err(Box::new)?;
                WsEvent::schema_variant_created(ctx, schema_id, new_schema_variant)
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
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
        // get latest bindings and fire an event
        let bindings: Vec<si_frontend_types::FuncBinding> = FuncBinding::for_func_id(ctx, func_id)
            .await?
            .into_iter()
            .map(|binding| binding.into())
            .collect();
        let new_bindings: Vec<si_frontend_types::FuncBinding> =
            FuncBinding::for_func_id(ctx, new_func.id)
                .await?
                .into_iter()
                .map(|binding| binding.into())
                .collect();
        WsEvent::func_bindings_updated(
            ctx,
            si_frontend_types::FuncBindings { bindings },
            Self::compile_types_from_bindings(ctx, func_id).await?,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;

        WsEvent::func_bindings_updated(
            ctx,
            si_frontend_types::FuncBindings {
                bindings: new_bindings,
            },
            Self::compile_types_from_bindings(ctx, new_func.id).await?,
        )
        .await?
        .publish_on_commit(ctx)
        .await?;
        Ok(new_func)
    }

    /// For a given [`FuncId`], regardless of what kind of [`Func`] it is, look for all associated bindings and remove
    /// them from every currently attached [`SchemaVariant`]
    /// todo: remove once front end consumes new routes
    pub async fn detach_func_from_everywhere(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncAuthoringResult<()> {
        // first check for attribute prototypes
        let maybe_attribute_prototypes =
            AttributePrototype::list_ids_for_func_id(ctx, func_id).await?;
        for attribute_prototype in maybe_attribute_prototypes {
            save::reset_attribute_prototype(ctx, attribute_prototype, true).await?;
        }

        // then check for and remove authentication prototypes
        for schema_variant_id in
            SchemaVariant::list_schema_variant_ids_using_auth_func_id(ctx, func_id).await?
        {
            SchemaVariant::remove_authentication_prototype(ctx, func_id, schema_variant_id).await?;
        }

        // then check for and remove action prototypes
        for action_prototype_id in ActionPrototype::list_for_func_id(ctx, func_id).await? {
            ActionPrototype::remove(ctx, action_prototype_id).await?;
        }
        Ok(())
    }

    #[instrument(level = "info", name = "func.authoring.save_code", skip(ctx))]
    /// Save only the code for the given [`FuncId`]
    /// Returns an error if the [`Func`] is currently locked
    pub async fn save_code(
        ctx: &DalContext,
        func_id: FuncId,
        code: String,
    ) -> FuncAuthoringResult<()> {
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        func.error_if_locked()?;
        Func::modify_by_id(ctx, func.id, |func| {
            func.code_base64 = Some(general_purpose::STANDARD_NO_PAD.encode(code));

            Ok(())
        })
        .await?;
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
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
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

    /// Comiples return types based on the [`FuncBinding`] for the Func
    pub async fn compile_types_from_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncAuthoringResult<String> {
        Ok(FuncBinding::compile_types(ctx, func_id).await?)
    }
}

/// The result of creating a [`Func`] via [`FuncAuthoringClient::create_func`].
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatedFunc {
    /// The id of the created [`Func`].
    pub id: FuncId,
    /// The handler of the created [`Func`].
    pub handler: Option<String>,
    /// The [kind](FuncKind) of the created [`Func`].
    pub kind: FuncKind,
    /// The name of the created [`Func`].
    pub name: String,
    /// The code for the created [`Func`].
    pub code: Option<String>,
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AttributeOutputLocation {
    #[serde(rename_all = "camelCase")]
    OutputSocket { output_socket_id: OutputSocketId },
    #[serde(rename_all = "camelCase")]
    Prop { prop_id: PropId },
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateFuncOptions {
    #[serde(rename_all = "camelCase")]
    ActionOptions {
        schema_variant_id: SchemaVariantId,
        action_kind: ActionKind,
    },
    #[serde(rename_all = "camelCase")]
    AttributeOptions {
        output_location: AttributeOutputLocation,
    },
    #[serde(rename_all = "camelCase")]
    AuthenticationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    CodeGenerationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    QualificationOptions { schema_variant_id: SchemaVariantId },
}
