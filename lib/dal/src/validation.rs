use serde::{Deserialize, Serialize};
use serde_json::json;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use si_layer_cache::LayerDbError;
use std::collections::VecDeque;
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::backend::validation::ValidationRunResult;
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::layer_db_types::{ValidationContent, ValidationContentV1};
use crate::prop::PropError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    pk, schema::variant::SchemaVariantError, AttributeValue, AttributeValueId, ChangeSetError,
    Component, ComponentId, Func, FuncError, HistoryEventError, Prop, Timestamp,
};
use crate::{DalContext, TransactionsError};

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(String),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
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

pub type ValidationResult<T> = Result<T, ValidationError>;

pk!(ValidationOutputId);

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

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ValidationContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        // If validation node exists, replace it, else create a new one.
        let id = if let Some(existing_node) =
            Self::find_for_attribute_value_id(ctx, attribute_value_id).await?
        {
            let id = existing_node.id;

            let idx = workspace_snapshot.get_node_index_by_id(id).await?;

            let node_weight = workspace_snapshot
                .get_node_weight(idx)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::ValidationOutput)?;

            let mut new_node_weight =
                node_weight.new_with_incremented_vector_clock(ctx.change_set()?)?;

            new_node_weight.new_content_hash(hash)?;

            workspace_snapshot
                .add_node(NodeWeight::Content(new_node_weight))
                .await?;
            workspace_snapshot.replace_references(idx).await?;

            id
        } else {
            let change_set = ctx.change_set()?;
            let id = change_set.generate_ulid()?;
            let node_weight =
                NodeWeight::new_content(change_set, id, ContentAddress::ValidationOutput(hash))?;
            workspace_snapshot.add_node(node_weight).await?;

            workspace_snapshot
                .add_edge(
                    attribute_value_id,
                    EdgeWeight::new(change_set, EdgeWeightKind::ValidationOutput)?,
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
            workspace_snapshot
                .remove_node_by_id(ctx.change_set()?, validation_id)
                .await?;
        }

        Ok(())
    }
}

impl ValidationOutput {
    pub async fn get_format_for_attribute_value_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ValidationResult<Option<String>> {
        Ok(
            if let Some(prop_id) = AttributeValue::prop_id_for_id(ctx, attribute_value_id)
                .await
                .map_err(|e| ValidationError::AttributeValue(e.to_string()))?
            {
                let prop = Prop::get_by_id(ctx, prop_id).await?;
                prop.validation_format
            } else {
                None
            },
        )
    }

    /// If an attribute value is for a [Prop](Prop) that has a `validation_format`, run a validation
    /// for that format and the value passed in.
    pub async fn compute_for_attribute_value_and_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> ValidationResult<Option<ValidationOutput>> {
        let validation_format = if let Some(validation_format) =
            Self::get_format_for_attribute_value_id(ctx, attribute_value_id).await?
        {
            validation_format
        } else {
            return Ok(None);
        };

        let args = json!({
            "value": value,
            "validation_format": validation_format,
        });

        let output = match FuncBinding::create_and_execute(
            ctx,
            args,
            Func::find_intrinsic(ctx, IntrinsicFunc::Validation).await?,
            vec![],
        )
        .await
        {
            Ok((_, result)) => {
                let message = if let Some(raw_value) = result.value() {
                    let validation_result: ValidationRunResult =
                        serde_json::from_value(raw_value.clone())?;

                    validation_result.error
                } else {
                    None
                };

                let status = if message.is_none() {
                    ValidationStatus::Success
                } else {
                    ValidationStatus::Failure
                };

                ValidationOutput { status, message }
            }
            Err(err) => match err {
                FuncBindingError::FuncBackendResultFailure { kind, message, .. } => {
                    ValidationOutput {
                        status: ValidationStatus::Error,
                        message: Some(format!("{kind}: {message}")),
                    }
                }
                _ => return Err(err.into()),
            },
        };
        Ok(Some(output))
    }

    pub async fn list_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ValidationResult<Vec<(AttributeValueId, ValidationOutput)>> {
        let component = Component::get_by_id(ctx, component_id)
            .await
            .map_err(|e| ValidationError::Component(e.to_string()))?;
        let domain_av = component
            .domain_prop_attribute_value(ctx)
            .await
            .map_err(|e| ValidationError::Component(e.to_string()))?;

        let mut outputs = vec![];
        let mut queue = VecDeque::from(vec![domain_av]);
        while let Some(attribute_value_id) = queue.pop_front() {
            let maybe_validation_output =
                ValidationOutputNode::find_for_attribute_value_id(ctx, attribute_value_id)
                    .await?
                    .map(|node| node.validation);

            let children_av_ids =
                AttributeValue::get_child_av_ids_for_ordered_parent(ctx, attribute_value_id)
                    .await
                    .map_err(|e| ValidationError::AttributeValue(e.to_string()))?;

            queue.extend(children_av_ids);

            if let Some(validation_output) = maybe_validation_output {
                outputs.push((attribute_value_id, validation_output));
            }
        }

        Ok(outputs)
    }
}
