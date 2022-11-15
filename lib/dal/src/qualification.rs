use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use thiserror::Error;
use veritech_client::{QualificationSubCheck, QualificationSubCheckStatus};

use crate::{
    attribute::context::UNSET_ID_VALUE,
    component,
    func::{
        binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError},
        FuncMetadataView,
    },
    validation::ValidationError,
    ws_event::{WsEvent, WsPayload},
    Component, ComponentId, DalContext, QualificationPrototype, QualificationPrototypeId,
    StandardModel, SystemId,
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

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum QualificationSummaryError {
    #[error("pg error: {0}")]
    PgError(#[from] PgError),
    #[error("error loading component validations: {0}")]
    ComponentError(#[from] component::ComponentError),
}

pub type QualificationSummaryResult<T> = Result<T, QualificationSummaryError>;

impl QualificationSummary {
    pub async fn get_summary(ctx: &DalContext) -> QualificationSummaryResult<QualificationSummary> {
        let rows = ctx
            .txns()
            .pg()
            .query(GET_SUMMARY, &[ctx.read_tenancy(), ctx.visibility()])
            .await?;

        let mut components = Vec::new();
        let mut components_succeeded = 0;
        let mut components_failed = 0;
        for row in &rows {
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

            let total = row.get::<&str, i64>("total_qualifications") + i64::from(has_validation);
            let succeeded = row.get::<&str, i64>("succeeded") + i64::from(validation_passed);
            let failed =
                row.get::<&str, i64>("failed") + i64::from(has_validation && !validation_passed);

            let component = QualificationSummaryForComponent {
                component_id,
                component_name: row.try_get("component_name")?,
                total,
                succeeded,
                failed,
            };

            if failed > 0 {
                components_failed += 1;
            } else if succeeded == total {
                components_succeeded += 1;
            }

            components.push(component);
        }

        Ok(QualificationSummary {
            total: rows.len() as i64,
            succeeded: components_succeeded,
            failed: components_failed,
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
    /// `validation_errors` is a Vec of the prop name being validated + the `ValidationError` object
    pub fn new_for_validation_errors(
        validation_errors: Vec<(String, ValidationError)>,
    ) -> QualificationView {
        let sub_checks: Vec<QualificationSubCheck> = validation_errors
            .iter()
            .map(|(prop_name, error)| QualificationSubCheck {
                description: format!("validation failed for \"{}\": {}", prop_name, error.message),
                status: QualificationSubCheckStatus::Failure,
            })
            .collect();

        QualificationView {
            title: "All fields are valid".into(),
            output: vec![],
            description: None,
            link: None,
            result: Some(QualificationResult {
                success: sub_checks.is_empty(),
                title: None,
                link: None,
                sub_checks,
            }),
            prototype_id: None,
        }
    }

    pub fn new_for_qualification_prototype(
        prototype: QualificationPrototype,
        func_metadata: FuncMetadataView,
    ) -> QualificationView {
        QualificationView {
            title: func_metadata.display_name,
            description: func_metadata.description.map(Into::into),
            link: func_metadata.link.map(Into::into),
            output: vec![],
            result: None,
            prototype_id: Some(*prototype.id()),
        }
    }

    pub async fn new_for_func_binding_return_value(
        ctx: &DalContext,
        prototype: QualificationPrototype,
        func_metadata: FuncMetadataView,
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
            title: func_metadata.display_name,
            description: func_metadata.description.map(Into::into),
            link: func_metadata.link.map(Into::into),
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
        ctx: &DalContext,
        prototype_id: QualificationPrototypeId,
        component_id: ComponentId,
        system_id: SystemId,
    ) -> Self {
        WsEvent::new(
            ctx,
            WsPayload::CheckedQualifications(QualificationCheckId {
                prototype_id,
                component_id,
                system_id,
            }),
        )
    }
}
