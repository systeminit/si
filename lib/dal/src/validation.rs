use itertools::join;
use joi_validator::Validator;
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use si_layer_cache::LayerDbError;
use std::collections::VecDeque;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::func::backend::validation::ValidationRunResult;
use crate::func::runner::{FuncRunner, FuncRunnerError};
use crate::layer_db_types::{ValidationContent, ValidationContentV1};
use crate::prop::PropError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    schema::variant::SchemaVariantError, AttributeValue, AttributeValueId, ChangeSetError,
    Component, ComponentId, FuncError, HistoryEventError, Timestamp,
};
use crate::{ComponentError, DalContext, TransactionsError};

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func run went away before a value could be sent down the channel")]
    FuncRunGone,
    #[error("func runner error: {0}")]
    FuncRunner(#[from] Box<FuncRunnerError>),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid prop id")]
    InvalidPropId,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("no prop or no validation format on the prop for attribute value: {0}")]
    NoValidationFormatForAttributeValue(AttributeValueId),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

impl From<AttributeValueError> for ValidationError {
    fn from(e: AttributeValueError) -> Self {
        Box::new(e).into()
    }
}

impl From<ComponentError> for ValidationError {
    fn from(e: ComponentError) -> Self {
        Box::new(e).into()
    }
}

impl From<FuncRunnerError> for ValidationError {
    fn from(e: FuncRunnerError) -> Self {
        Box::new(e).into()
    }
}

pub type ValidationResult<T> = Result<T, ValidationError>;

pub use si_id::ValidationOutputId;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Pending,
    Error,
    Failure,
    Success,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationOutput {
    pub status: ValidationStatus,
    pub message: Option<String>,
}

/// Stores the validation output for an [AttributeValue]. Should only exist if
/// the av is for a prop and that prop has a validation format bound to it.
///
/// Its only relation is established as follows:
///
/// [AttributeValue] -- [EdgeWeightKind::ValidationOutput] --> [ValidationOutputNode]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationOutputNode {
    pub id: ValidationOutputId,
    pub validation: ValidationOutput,
}

impl ValidationOutputNode {
    pub async fn upsert_or_wipe_for_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        maybe_validation_output: Option<ValidationOutput>,
    ) -> ValidationResult<Option<Self>> {
        // If no validation format, wipe any validation results
        if ValidationOutput::get_format_for_attribute_value_id(ctx, attribute_value_id)
            .await?
            .is_none()
        {
            Self::wipe_for_attribute_value_id(ctx, attribute_value_id).await?;
            return Ok(None);
        }

        // If no validation output, wipe any validation results
        let Some(validation) = maybe_validation_output else {
            Self::wipe_for_attribute_value_id(ctx, attribute_value_id).await?;
            return Ok(None);
        };

        // Now we're sure we're creating something, compute content
        let timestamp = Timestamp::now();

        let content = ValidationContentV1 {
            timestamp,
            status: validation.status,
            message: validation.message.clone(),
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ValidationContent::V1(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        // If validation node exists, replace it, else create a new one.
        let id = if let Some(existing_node) =
            Self::find_for_attribute_value_id(ctx, attribute_value_id).await?
        {
            let id = existing_node.id;

            let node_weight = workspace_snapshot
                .get_node_weight(id)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::ValidationOutput)?;

            let mut new_node_weight = node_weight.clone();

            new_node_weight.new_content_hash(hash)?;

            workspace_snapshot
                .add_or_replace_node(NodeWeight::Content(new_node_weight))
                .await?;

            id
        } else {
            let id = workspace_snapshot.generate_ulid().await?;
            let lineage_id = workspace_snapshot.generate_ulid().await?;
            let node_weight =
                NodeWeight::new_content(id, lineage_id, ContentAddress::ValidationOutput(hash));
            workspace_snapshot.add_or_replace_node(node_weight).await?;

            workspace_snapshot
                .add_edge(
                    attribute_value_id,
                    EdgeWeight::new(EdgeWeightKind::ValidationOutput),
                    id,
                )
                .await?;
            id.into()
        };

        Ok(Some(Self { id, validation }))
    }

    pub async fn find_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ValidationResult<Option<ValidationOutputNode>> {
        let v = ctx
            .workspace_snapshot()?
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::ValidationOutput,
            )
            .await?;

        if let Some(validation_idx) = v.first() {
            let node_weight = ctx
                .workspace_snapshot()?
                .get_node_weight(*validation_idx)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::ValidationOutput)?;

            let id = node_weight.id();

            let ValidationContent::V1(ValidationContentV1 {
                status, message, ..
            }) = ctx
                .layer_db()
                .cas()
                .try_read_as(&node_weight.content_hash())
                .await?
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(id))?;

            Ok(Some(Self {
                id: id.into(),
                validation: ValidationOutput { status, message },
            }))
        } else {
            Ok(None)
        }
    }

    async fn wipe_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ValidationResult<()> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        for validation_idx in workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                attribute_value_id,
                EdgeWeightKindDiscriminants::ValidationOutput,
            )
            .await?
        {
            let validation_id = workspace_snapshot
                .get_node_weight(validation_idx)
                .await?
                .id();
            workspace_snapshot.remove_node_by_id(validation_id).await?;
        }

        Ok(())
    }
}

impl ValidationOutput {
    pub async fn get_format_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ValidationResult<Option<String>> {
        Ok(AttributeValue::prop_opt(ctx, attribute_value_id)
            .await?
            .and_then(|prop| prop.validation_format))
    }

    /// If an attribute value is for a [Prop](Prop) that has a `validation_format`, run a validation
    /// for that format.
    pub async fn compute_for_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ValidationResult<Option<ValidationOutput>> {
        let value = AttributeValue::get_by_id(ctx, attribute_value_id)
            .await?
            .value_or_default(ctx)
            .await?;

        match Self::get_format_for_attribute_value_id(ctx, attribute_value_id).await? {
            None => Ok(None),
            Some(validation_format) => Ok(Some(
                // If we can't deserialize the validation format, run remotely
                match serde_json::from_str(&validation_format) {
                    Ok(validator) => run_locally(validator, value),
                    Err(serde_error) => {
                        run_remotely(
                            ctx,
                            attribute_value_id,
                            value,
                            validation_format,
                            Some(serde_error),
                        )
                        .await?
                    }
                },
            )),
        }
    }

    pub async fn list_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ValidationResult<Vec<(AttributeValueId, ValidationOutput)>> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let domain_av = component.domain_prop_attribute_value(ctx).await?;

        let mut outputs = vec![];
        let mut queue = VecDeque::from(vec![domain_av]);
        while let Some(attribute_value_id) = queue.pop_front() {
            let maybe_validation_output =
                ValidationOutputNode::find_for_attribute_value_id(ctx, attribute_value_id)
                    .await?
                    .map(|node| node.validation);

            let children_av_ids =
                AttributeValue::get_child_av_ids_in_order(ctx, attribute_value_id).await?;

            queue.extend(children_av_ids);

            if let Some(validation_output) = maybe_validation_output {
                outputs.push((attribute_value_id, validation_output));
            }
        }

        Ok(outputs)
    }
}

#[instrument(
    name = "validation.run_locally",
    level = "info",
    skip_all,
    fields(si.validation.rules = %validator.rule_names().join(", "))
)]
fn run_locally(validator: Validator, value: Option<serde_json::Value>) -> ValidationOutput {
    // We treat value as undefined if it's null
    let value = match value {
        Some(serde_json::Value::Null) => None,
        value => value,
    };
    match validator.validate(&value).error {
        None => ValidationOutput {
            status: ValidationStatus::Success,
            message: None,
        },
        Some(error) => ValidationOutput {
            status: ValidationStatus::Failure,
            message: Some(join(error.details.iter().map(|e| &e.message), "\n")),
        },
    }
}

#[instrument(
    name = "validation.run_remotely",
    level = "info",
    skip_all,
    fields(si.validation.because = because.map(|e| e.to_string()))
)]
async fn run_remotely(
    ctx: &DalContext,
    attribute_value_id: AttributeValueId,
    value: Option<serde_json::Value>,
    validation_format: String,
    because: Option<serde_json::Error>,
) -> ValidationResult<ValidationOutput> {
    let result_channel =
        FuncRunner::run_validation_format(ctx, attribute_value_id, value, validation_format)
            .await?;

    let func_result_value = match result_channel
        .await
        .map_err(|_| ValidationError::FuncRunGone)?
    {
        Ok(func_run_result) => func_run_result,
        Err(FuncRunnerError::ResultFailure { kind, message, .. }) => {
            return Ok(ValidationOutput {
                status: ValidationStatus::Error,
                message: Some(format!("{kind}: {message}")),
            });
        }
        Err(e) => return Err(e.into()),
    };

    let message = match func_result_value.value() {
        Some(raw_value) => serde_json::from_value::<ValidationRunResult>(raw_value.clone())?.error,
        None => None,
    };

    ctx.layer_db()
        .func_run()
        .set_state_to_success(
            func_result_value.func_run_id(),
            ctx.events_tenancy(),
            ctx.events_actor(),
        )
        .await?;

    Ok(ValidationOutput {
        status: match message {
            Some(_) => ValidationStatus::Failure,
            None => ValidationStatus::Success,
        },
        message,
    })
}
