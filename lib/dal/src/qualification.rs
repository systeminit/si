use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::component::qualification::QualificationEntry;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::{
    func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    ws_event::{WsEvent, WsPayload},
    Component, ComponentError, ComponentId, DalContext, FuncId, StandardModel, StandardModelError,
    WsEventResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationSummaryForComponent {
    component_id: ComponentId,
    component_name: String,
    total: i64,
    warned: i64,
    succeeded: i64,
    failed: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationSummary {
    total: i64,
    succeeded: i64,
    warned: i64,
    failed: i64,
    components: Vec<QualificationSummaryForComponent>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum QualificationSummaryError {
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
}

pub type QualificationSummaryResult<T> = Result<T, QualificationSummaryError>;

impl QualificationSummary {
    // TODO(nick): turn this into a query for performance. The original version leveraged a query,
    // but since qualifications are now on the prop tree, there is no longer a relevant query
    // to help here. I'd write it, but the PR replacing the prototypes and resolvers with the prop
    // tree is getting huge.
    #[instrument(skip_all)]
    pub async fn get_summary(ctx: &DalContext) -> QualificationSummaryResult<QualificationSummary> {
        let mut component_summaries = Vec::new();
        let mut components_succeeded = 0;
        let mut components_warned = 0;
        let mut components_failed = 0;
        let mut total = 0;

        for component in Component::list(ctx).await? {
            let component_id = *component.id();
            let qualifications = Component::list_qualifications(ctx, component_id).await?;

            let individual_total = qualifications.len() as i64;
            let mut succeeded = 0;
            let mut warned = 0;
            let mut failed = 0;
            for qualification in qualifications {
                if let Some(result) = qualification.result {
                    match result.status {
                        QualificationSubCheckStatus::Success => succeeded += 1,
                        QualificationSubCheckStatus::Warning => warned += 1,
                        QualificationSubCheckStatus::Failure => failed += 1,
                        QualificationSubCheckStatus::Unknown => {}
                    }
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

#[derive(Error, Debug)]
pub enum QualificationError {
    #[error("function binding return value error: {0}")]
    FuncBindingReturnValueError(#[from] FuncBindingReturnValueError),
    #[error("no value returned in qualification function result")]
    NoValue,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
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
}

impl PartialOrd for QualificationView {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.title.partial_cmp(&other.title)
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
        qualification_name: &str,
        qualification_entry: QualificationEntry,
        attribute_prototype_func_id: FuncId,
        func_binding_return_value_id: FuncBindingReturnValueId,
    ) -> Result<Option<Self>, QualificationError> {
        let func_binding_return_value =
            FuncBindingReturnValue::get_by_id(ctx, &func_binding_return_value_id)
                .await?
                .ok_or(FuncBindingReturnValueError::NotFound(
                    func_binding_return_value_id,
                ))?;

        // If the func binding return value on this does not match the prototype func, it means
        // the qualification has not yet been run
        if *func_binding_return_value.func_id() != attribute_prototype_func_id {
            return Ok(None);
        }

        let func_metadata = func_binding_return_value.func_metadata_view(ctx).await?;

        let output_streams = func_binding_return_value.get_output_stream(ctx).await?;
        let output = match output_streams {
            Some(streams) => streams
                .into_iter()
                .map(|output_stream| QualificationOutputStreamView {
                    stream: output_stream.stream,
                    line: output_stream.message,
                    level: output_stream.level,
                })
                .collect::<Vec<QualificationOutputStreamView>>(),
            None => Vec::with_capacity(0),
        };

        let sub_check = QualificationSubCheck {
            description: match qualification_entry.message {
                Some(message) => message,
                None => String::from("no description provided"),
            },
            status: qualification_entry
                .result
                .unwrap_or(QualificationSubCheckStatus::Unknown),
        };
        let result = Some(QualificationResult {
            status: qualification_entry
                .result
                .unwrap_or(QualificationSubCheckStatus::Unknown),
            title: Some(func_metadata.display_name.clone()),
            link: None,
            sub_checks: vec![sub_check],
        });

        Ok(Some(QualificationView {
            title: func_metadata.display_name,
            description: func_metadata.description.map(Into::into),
            link: func_metadata.link.map(Into::into),
            output,
            result,
            qualification_name: qualification_name.to_string(),
        }))
    }
}

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
    Success,
    Warning,
    Failure,
    #[default]
    Unknown,
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
