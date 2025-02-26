use anyhow::Result;
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_layer_cache::LayerDbError;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;
use crate::component::qualification::QualificationEntry;
use crate::func::FuncError;
use crate::prop::PropError;
use crate::validation::{ValidationError, ValidationOutput, ValidationStatus};
use crate::AttributeValue;
use crate::{
    ws_event::{WsEvent, WsPayload},
    Component, ComponentError, ComponentId, DalContext, Prop, StandardModelError, WsEventResult,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct QualificationSummaryForComponent {
    pub component_id: ComponentId,
    pub component_name: String,
    pub total: i64,
    pub warned: i64,
    pub succeeded: i64,
    pub failed: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QualificationSummary {
    pub total: i64,
    pub succeeded: i64,
    pub warned: i64,
    pub failed: i64,
    pub components: Vec<QualificationSummaryForComponent>,
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum QualificationSummaryError {
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type QualificationSummaryResult<T> = Result<T>;

impl QualificationSummary {
    #[instrument(level = "debug", skip_all)]
    pub async fn get_summary(ctx: &DalContext) -> QualificationSummaryResult<QualificationSummary> {
        let mut components_succeeded = 0;
        let mut components_warned = 0;
        let mut components_failed = 0;
        let mut total = 0;

        let mut component_summaries = vec![];

        for component in Component::list(ctx).await? {
            let component_id = component.id();
            let qualification_statuses =
                Component::list_qualification_statuses(ctx, component_id).await?;

            let individual_total = qualification_statuses.len() as i64;
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

            let individual_summary = QualificationSummaryForComponent {
                component_id,
                component_name: component.name(ctx).await?,
                total: individual_total,
                succeeded,
                warned,
                failed,
            };

            // Update counters for all components.
            if failed > 0 {
                components_failed += 1;
            } else if warned > 0 {
                components_warned += 1;
            } else {
                components_succeeded += 1;
            }
            total += individual_total;

            component_summaries.push(individual_summary);
        }

        Ok(QualificationSummary {
            total,
            succeeded: components_succeeded,
            warned: components_warned,
            failed: components_failed,
            components: component_summaries,
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
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
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
        attribute_value: AttributeValue,
    ) -> QualificationSummaryResult<Option<Self>> {
        let maybe_qual_run = ctx
            .layer_db()
            .func_run()
            .get_last_qualification_for_attribute_value_id(
                ctx.events_tenancy().workspace_pk,
                attribute_value.id(),
            )
            .await?;
        match maybe_qual_run {
            Some(qual_run) => {
                let qualification_entry: QualificationEntry =
                    match attribute_value.view(ctx).await? {
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

                let (output, finalized) = match ctx
                    .layer_db()
                    .func_run_log()
                    .get_for_func_run_id(qual_run.id())
                    .await?
                {
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
    ) -> QualificationSummaryResult<Option<Self>> {
        let mut output = Vec::new();

        let mut status = QualificationSubCheckStatus::Success;

        let mut fail_counter = 0;

        // Note(victor): If this is ever the bottleneck, we could pretty easily compute a
        // validations summary for a component and store it on the graph during the
        // compute_validations job.
        // Then we'd just load it here and convert to the view struct
        for (av_id, validation_output) in
            ValidationOutput::list_for_component(ctx, component_id).await?
        {
            if validation_output.status != ValidationStatus::Success {
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
