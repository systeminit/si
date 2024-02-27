use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::component::qualification::QualificationEntry;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::{
    func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
    ws_event::{WsEvent, WsPayload},
    ComponentError, ComponentId, DalContext, FuncId, Prop, StandardModel, StandardModelError,
    ValidationResolver, ValidationResolverError, ValidationStatus, WsEventResult,
};
use crate::{standard_model, TransactionsError};

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
    #[error(transparent)]
    Transaction(#[from] TransactionsError),
}

pub type QualificationSummaryResult<T> = Result<T, QualificationSummaryError>;

impl QualificationSummary {
    #[instrument(level = "debug", skip_all)]
    pub async fn get_summary(ctx: &DalContext) -> QualificationSummaryResult<QualificationSummary> {
        let mut components_succeeded = 0;
        let mut components_warned = 0;
        let mut components_failed = 0;
        let mut total = 0;

        let qualification_summary_for_components: Vec<QualificationSummaryForComponent> =
            standard_model::list(ctx, "summary_qualifications").await?;
        for component_summary in qualification_summary_for_components.iter() {
            components_succeeded += component_summary.succeeded;
            components_warned += component_summary.warned;
            components_failed += component_summary.failed;
            total += 1;
        }

        Ok(QualificationSummary {
            total,
            succeeded: components_succeeded,
            warned: components_warned,
            failed: components_failed,
            components: qualification_summary_for_components,
        })
    }
}

#[remain::sorted]
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
    #[error(transparent)]
    ValidationResolver(#[from] ValidationResolverError),
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

    pub async fn new_for_validations(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> Result<Option<Self>, QualificationError> {
        let mut output = Vec::new();

        let mut status = QualificationSubCheckStatus::Success;

        let mut fail_counter = 0;
        for resolver in ValidationResolver::find_by_attr(ctx, "component_id", &component_id).await?
        {
            let value = resolver.value()?;
            if value.status != ValidationStatus::Success {
                status = QualificationSubCheckStatus::Failure;
                fail_counter += 1;

                if let Some(prop) = Prop::get_by_id(ctx, &resolver.prop_id()).await? {
                    output.push(QualificationOutputStreamView {
                        stream: "stdout".to_owned(),
                        level: "log".to_owned(),
                        line: format!("{}: {}", prop.name(), value.message),
                    });
                }
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
            title: "Schema Validations".to_owned(),
            description: None,
            link: None,
            output,
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
