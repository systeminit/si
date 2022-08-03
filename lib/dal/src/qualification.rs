use serde::{Deserialize, Serialize};

use thiserror::Error;
use veritech::QualificationSubCheck;

use crate::attribute::context::UNSET_ID_VALUE;
use crate::func::backend::validation::ValidationError;
use crate::func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
use crate::ws_event::{WsEvent, WsPayload};
use crate::{
    component, BillingAccountId, Component, ComponentId, DalContext, HistoryActor,
    QualificationPrototype, QualificationPrototypeId, StandardModel, SystemId,
};

const GET_SUMMARY: &str = include_str!("queries/qualifications_summary_for_tenancy.sql");

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationSummaryForComponent {
    component_id: ComponentId,
    component_name: String,
    total: i64,
    succeeded: i64,
    failed: i64,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QualificationSummary {
    total: i64,
    succeeded: i64,
    failed: i64,
    components: Vec<QualificationSummaryForComponent>,
}

#[derive(Error, Debug)]
pub enum QualificationSummaryError {
    #[error("error accessing database")]
    PgError(#[from] tokio_postgres::Error),
    #[error("error loading component validations")]
    ComponentError(#[from] Box<component::ComponentError>),
}

pub type QualificationSummaryResult<T> = Result<T, QualificationSummaryError>;

impl QualificationSummary {
    pub async fn get_summary(
        ctx: &DalContext<'_, '_>,
    ) -> QualificationSummaryResult<QualificationSummary> {
        let rows = ctx
            .txns()
            .pg()
            .query(GET_SUMMARY, &[ctx.read_tenancy(), ctx.visibility()])
            .await?;

        let mut components = Vec::new();
        let mut total = 0;
        let mut succeeded = 0;
        let mut failed = 0;
        for row in rows {
            let component_id = row.try_get("component_id")?;

            let (has_validation, validation_passed) =
                match Component::list_validations_as_qualification_for_component_id(
                    ctx,
                    component_id,
                    UNSET_ID_VALUE.into(),
                )
                .await?
                .result
                {
                    None => (false, false),
                    Some(qual_result) => (true, qual_result.success),
                };

            let component_total =
                row.get::<&str, i64>("total_qualifications") + if has_validation { 1 } else { 0 };
            let component_succeeded =
                row.get::<&str, i64>("succeeded") + if validation_passed { 1 } else { 0 };
            let component_failed = row.get::<&str, i64>("failed")
                + if has_validation && !validation_passed {
                    1
                } else {
                    0
                };

            let component = QualificationSummaryForComponent {
                component_id,
                component_name: row.try_get("component_name")?,
                total: component_total,
                succeeded: component_succeeded,
                failed: component_failed,
            };

            total += component.total;
            succeeded += component.succeeded;
            failed += component.failed;

            components.push(component);
        }

        Ok(QualificationSummary {
            total,
            succeeded,
            failed,
            components,
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
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationResult {
    pub success: bool,
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
    /// The validations qualification doesn't need a prototype, but it can't be edited
    pub prototype_id: Option<QualificationPrototypeId>,
}

impl QualificationView {
    pub fn new_for_validation_errors(validation_errors: Vec<ValidationError>) -> QualificationView {
        let mut sub_checks: Vec<QualificationSubCheck> = Vec::new();
        let mut success = true;
        for validation_error in validation_errors {
            let description = format!("field validation failed: {}", validation_error.message);
            sub_checks.push(QualificationSubCheck {
                description,
                status: veritech::QualificationSubCheckStatus::Failure,
            });
            success = false;
        }
        QualificationView {
            title: "All fields are valid".into(),
            output: vec![],
            description: None,
            link: None,
            result: Some(QualificationResult {
                success,
                title: None,
                link: None,
                sub_checks,
            }),
            prototype_id: None,
        }
    }

    pub fn new_for_qualification_prototype(prototype: QualificationPrototype) -> QualificationView {
        QualificationView {
            title: prototype.title().into(),
            description: prototype.description().map(Into::into),
            link: prototype.link().map(Into::into),
            output: vec![],
            result: None,
            prototype_id: Some(*prototype.id()),
        }
    }

    pub async fn new_for_func_binding_return_value(
        ctx: &DalContext<'_, '_>,
        prototype: QualificationPrototype,
        func_binding_return_value: FuncBindingReturnValue,
    ) -> Result<Self, QualificationError> {
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

        Ok(QualificationView {
            title: prototype.title().into(),
            description: prototype.description().map(Into::into),
            link: prototype.link().map(Into::into),
            output,
            result: func_binding_return_value
                .value()
                .map(|json| serde_json::from_value(json.clone()))
                .transpose()?,
            prototype_id: Some(*prototype.id()),
        })
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckId {
    prototype_id: QualificationPrototypeId,
    component_id: ComponentId,
    system_id: SystemId,
}

impl WsEvent {
    pub fn checked_qualifications(
        prototype_id: QualificationPrototypeId,
        component_id: ComponentId,
        system_id: SystemId,
        billing_account_ids: Vec<BillingAccountId>,
        history_actor: &HistoryActor,
    ) -> Self {
        WsEvent::new(
            billing_account_ids,
            history_actor.clone(),
            WsPayload::CheckedQualifications(QualificationCheckId {
                prototype_id,
                component_id,
                system_id,
            }),
        )
    }
}
