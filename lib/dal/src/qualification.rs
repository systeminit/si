use crate::DalContext;
use serde::{Deserialize, Serialize};

use thiserror::Error;
use veritech::QualificationSubCheck;

use crate::func::backend::validation::ValidationError;
use crate::func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
use crate::ws_event::{WsEvent, WsPayload};
use crate::{BillingAccountId, ComponentId, HistoryActor, Prop, QualificationPrototype, SystemId};

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
    pub sub_checks: Option<Vec<QualificationSubCheck>>,
}

/// A view on "OutputStream" from cyclone.
#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationOutputStreamView {
    pub stream: String,
    pub line: String,
    pub level: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct QualificationView {
    pub title: String,
    /// A collection of "OutputStream" views from cyclone.
    pub output: Vec<QualificationOutputStreamView>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub result: Option<QualificationResult>,
}

impl QualificationView {
    pub fn new_for_validation_errors(
        prop_validation_errors: Vec<(Prop, Vec<ValidationError>)>,
    ) -> QualificationView {
        let mut sub_checks: Vec<QualificationSubCheck> = Vec::new();
        let mut success = true;
        for (prop, validation_errors) in prop_validation_errors.into_iter() {
            for validation_error in validation_errors.into_iter() {
                let description =
                    format!("field {} failed: {}", prop.name(), validation_error.message);
                sub_checks.push(QualificationSubCheck {
                    description,
                    status: veritech::QualificationSubCheckStatus::Failure,
                });
                success = false;
            }
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
                sub_checks: Some(sub_checks),
            }),
        }
    }

    pub fn new_for_qualification_prototype(prototype: QualificationPrototype) -> QualificationView {
        QualificationView {
            title: prototype.title().into(),
            description: None,
            link: None,
            output: vec![],
            result: None,
        }
    }

    pub async fn new_for_func_binding_return_value(
        ctx: &DalContext<'_, '_>,
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

        if let Some(qual_result_json) = func_binding_return_value.value() {
            let result = serde_json::from_value(qual_result_json.clone())?;
            Ok(QualificationView {
                title: "Unknown (no title provided)".to_string(),
                output,
                description: None,
                link: None,
                result: Some(result),
            })
        } else {
            Ok(QualificationView {
                title: "Unknown (no title provided)".to_string(),
                output,
                description: None,
                link: None,
                result: None,
            })
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QualificationCheckId {
    component_id: ComponentId,
    system_id: SystemId,
}

impl WsEvent {
    pub fn checked_qualifications(
        component_id: ComponentId,
        system_id: SystemId,
        billing_account_ids: Vec<BillingAccountId>,
        history_actor: &HistoryActor,
    ) -> Self {
        WsEvent::new(
            billing_account_ids,
            history_actor.clone(),
            WsPayload::CheckedQualifications(QualificationCheckId {
                component_id,
                system_id,
            }),
        )
    }
}
