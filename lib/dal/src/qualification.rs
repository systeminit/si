use serde::{Deserialize, Serialize};

use thiserror::Error;
use veritech::QualificationSubCheck;

use crate::func::backend::validation::ValidationError;
use crate::func::binding_return_value::{FuncBindingReturnValue, FuncBindingReturnValueError};
use crate::ws_event::{WsEvent, WsPayload};
use crate::{
    BillingAccountId, ComponentId, DalContext, HistoryActor, QualificationPrototype,
    QualificationPrototypeId, StandardModel, SystemId,
};

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

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
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
