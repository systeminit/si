use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgError;
use si_db::{
    FuncRunDb,
    FuncRunLogDb,
};
use si_frontend_types::ComponentQualificationStats;
use si_id::AttributeValueId;
use si_layer_cache::LayerDbError;
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributeValue,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    Prop,
    WsEventResult,
    attribute::value::AttributeValueError,
    component::qualification::QualificationEntry,
    func::FuncError,
    prop::PropError,
    validation::{
        ValidationError,
        ValidationOutput,
        ValidationStatus,
    },
    ws_event::{
        WsEvent,
        WsPayload,
    },
};

#[derive(Deserialize, Serialize, Debug)]
pub struct QualificationSummaryForComponent {
    pub component_id: ComponentId,
    pub component_name: String,
    pub total: u64,
    pub warned: u64,
    pub succeeded: u64,
    pub failed: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QualificationSummary {
    pub total: u64,
    pub succeeded: u64,
    pub warned: u64,
    pub failed: u64,
    pub components: Vec<QualificationSummaryForComponent>,
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum QualificationSummaryError {
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
}

impl From<ComponentError> for QualificationSummaryError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

pub type QualificationSummaryResult<T> = Result<T, QualificationSummaryError>;

impl QualificationSummary {
    #[instrument(level = "debug", skip_all)]
    pub async fn get_summary(ctx: &DalContext) -> QualificationSummaryResult<QualificationSummary> {
        let mut components_succeeded = 0;
        let mut components_warned = 0;
        let mut components_failed = 0;
        let mut total = 0;

        let component_ids = Component::list_ids(ctx).await?;
        let mut component_summaries = Vec::with_capacity(component_ids.len());

        for component_id in component_ids {
            let stats = Self::individual_stats(ctx, component_id).await?;

            // Update counters for all components.
            if stats.failed > 0 {
                components_failed += 1;
            } else if stats.warned > 0 {
                components_warned += 1;
            } else {
                components_succeeded += 1;
            }
            total += stats.total;

            component_summaries.push(QualificationSummaryForComponent {
                component_id,
                component_name: Component::name_by_id(ctx, component_id).await?,
                total: stats.total,
                warned: stats.warned,
                succeeded: stats.succeeded,
                failed: stats.failed,
            });
        }

        Ok(QualificationSummary {
            total,
            succeeded: components_succeeded,
            warned: components_warned,
            failed: components_failed,
            components: component_summaries,
        })
    }

    #[instrument(level = "debug", skip_all)]
    pub async fn individual_stats(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> QualificationSummaryResult<ComponentQualificationStats> {
        let qualification_statuses =
            Component::list_qualification_statuses(ctx, component_id).await?;

        let total = qualification_statuses.len() as u64;

        let mut succeeded = 0;
        let mut warned = 0;
        let mut failed = 0;

        for status in qualification_statuses.iter().flatten() {
            match status {
                QualificationSubCheckStatus::Success => succeeded += 1,
                QualificationSubCheckStatus::Warning => warned += 1,
                QualificationSubCheckStatus::Failure => failed += 1,
                QualificationSubCheckStatus::Unknown => {}
            }
        }

        // FIXME(nick): delete this when we switch to the new UI.
        let running = total
            .checked_sub(warned)
            .and_then(|v| v.checked_sub(succeeded))
            .and_then(|v| v.checked_sub(failed))
            .unwrap_or(0);

        Ok(ComponentQualificationStats {
            total,
            warned,
            succeeded,
            failed,
            running,
        })
    }
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum QualificationError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("no value returned in qualification function result")]
    NoValue,
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::SiDbError),
    #[error("validation resolver error: {0}")]
    ValidationResolver(#[from] ValidationError),
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationResult {
    pub status: QualificationSubCheckStatus,
    pub title: Option<String>,
    pub link: Option<String>,
    pub sub_checks: Vec<QualificationSubCheck>,
}

/// A view on "OutputStream" from cyclone.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationOutputStreamView {
    pub stream: String,
    pub line: String,
    pub level: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QualificationView {
    pub title: String,
    /// A collection of "OutputStream" views from cyclone.
    pub output: Vec<QualificationOutputStreamView>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub result: Option<QualificationResult>,
    pub qualification_name: String,
    pub finalized: bool,
}

impl PartialOrd for QualificationView {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QualificationView {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.title.cmp(&other.title)
    }
}

impl QualificationView {
    pub async fn new(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> Result<Option<Self>, QualificationError> {
        let maybe_qual_run = FuncRunDb::get_last_qualification_for_attribute_value_id(
            ctx,
            ctx.events_tenancy().workspace_pk,
            attribute_value_id,
        )
        .await?;
        match maybe_qual_run {
            Some(qual_run) => {
                let qualification_entry: QualificationEntry =
                    match AttributeValue::view(ctx, attribute_value_id).await? {
                        Some(value) => serde_json::from_value(value)?,
                        None => return Ok(None),
                    };

                let sub_check = QualificationSubCheck {
                    description: qualification_entry
                        .message
                        .unwrap_or_else(|| "no description provided".to_string()),
                    status: qualification_entry
                        .result
                        .unwrap_or(QualificationSubCheckStatus::Unknown),
                };
                let result = Some(QualificationResult {
                    status: qualification_entry
                        .result
                        .unwrap_or(QualificationSubCheckStatus::Unknown),
                    title: Some(qual_run.function_name().to_string()),
                    link: qual_run.function_link().map(str::to_string),
                    sub_checks: vec![sub_check],
                });

                let (output, finalized) =
                    match FuncRunLogDb::get_for_func_run_id(ctx, qual_run.id()).await? {
                        Some(func_run_logs) => {
                            let output = func_run_logs
                                .logs()
                                .iter()
                                .map(|l| QualificationOutputStreamView {
                                    stream: l.stream.clone(),
                                    line: l.message.clone(),
                                    level: l.level.clone(),
                                })
                                .collect();
                            let finalized = func_run_logs.is_finalized();

                            (output, finalized)
                        }
                        None => (Vec::new(), false),
                    };

                Ok(Some(QualificationView {
                    title: qual_run
                        .function_display_name()
                        .map(str::to_string)
                        .unwrap_or_else(|| qual_run.function_name().to_string()),
                    description: qual_run.function_description().map(str::to_string),
                    link: qual_run.function_link().map(str::to_string),
                    output,
                    finalized,
                    result,
                    qualification_name: qual_run.function_name().to_string(),
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn new_for_validations(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> Result<Option<Self>, QualificationError> {
        let mut status = QualificationSubCheckStatus::Success;

        let mut fail_counter = 0;
        let mut has_active_validations = false;

        // Note(victor): If this is ever the bottleneck, we could pretty easily compute a
        // validations summary for a component and store it on the graph during the
        // compute_validations job.
        // Then we'd just load it here and convert to the view struct
        let component_validation_outputs =
            ValidationOutput::list_for_component(ctx, component_id).await?;
        let mut output = Vec::with_capacity(component_validation_outputs.len());
        for (av_id, validation_output) in component_validation_outputs {
            // We have validations therefore, we need to show the validations in the Qualifications output
            has_active_validations = true;
            if validation_output.status != ValidationStatus::Success {
                // We need to filter out any false positive results for subscriptions that
                // have yet to propagate their value during DVU - this would be a misleading
                // result for a user
                let av = AttributeValue::get_by_id(ctx, av_id).await?;
                if AttributeValue::subscriptions(ctx, av_id).await?.is_some()
                    && av.value(ctx).await?.is_none()
                {
                    continue;
                }

                status = QualificationSubCheckStatus::Failure;
                fail_counter += 1;

                let prop_id = AttributeValue::prop_id(ctx, av_id).await?;
                let prop = Prop::get_by_id(ctx, prop_id).await?;

                output.push(QualificationOutputStreamView {
                    stream: "stdout".to_owned(),
                    level: "log".to_owned(),
                    line: format!(
                        "{}: {}",
                        prop.name,
                        validation_output
                            .message
                            .clone()
                            .unwrap_or("message missing".to_string())
                    ),
                });
            }
        }

        if !has_active_validations {
            return Ok(None);
        }

        let result = Some(QualificationResult {
            status,
            title: None,
            link: None,
            sub_checks: vec![QualificationSubCheck {
                description: format!("Component has {fail_counter} invalid value(s)."),
                status,
            }],
        });

        Ok(Some(QualificationView {
            title: "Prop Validations".to_owned(),
            description: None,
            link: None,
            output,
            finalized: true,
            result,
            qualification_name: "validations".to_owned(),
        }))
    }
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
    Copy,
    Default,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum QualificationSubCheckStatus {
    Failure,
    Success,
    #[default]
    Unknown,
    Warning,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationSubCheck {
    pub description: String,
    pub status: QualificationSubCheckStatus,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckPayload {
    component_id: ComponentId,
}

impl WsEvent {
    pub async fn checked_qualifications(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::CheckedQualifications(QualificationCheckPayload { component_id }),
        )
        .await
    }
}
