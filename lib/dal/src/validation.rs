use serde::{Deserialize, Serialize};
use serde_json::json;
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::backend::validation::ValidationRunResult;
use crate::func::binding::{FuncBinding, FuncBindingError};
use crate::func::intrinsics::IntrinsicFunc;
use crate::prop::PropError;
use crate::{
    schema::variant::SchemaVariantError, AttributeValue, AttributeValueId, ComponentId, Func,
    FuncError, HistoryEventError, Prop, PropId,
};
use crate::{DalContext, TransactionsError};

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("attribute value error: {0}")]
    AttributeValue(String),
    #[error("component error: {0}")]
    Component(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("invalid prop id")]
    InvalidPropId,
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
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
}

pub type ValidationResult<T> = Result<T, ValidationError>;

#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Error,
    Failure,
    Success,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ValidationOutput {
    pub status: ValidationStatus,
    pub message: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Validation {
    prop_id: PropId,
    attribute_value_id: AttributeValueId,
    key: Option<String>,
    value: ValidationOutput,
}

impl Validation {
    pub fn value(&self) -> &ValidationOutput {
        &self.value
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub async fn compute_for_attribute_value(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<serde_json::Value>,
    ) -> ValidationResult<Option<Validation>> {
        if let Some(prop_id) = AttributeValue::prop_id_for_id(ctx, attribute_value_id)
            .await
            .map_err(|e| ValidationError::AttributeValue(e.to_string()))?
        {
            let prop = Prop::get_by_id(ctx, prop_id).await?;
            let maybe_validation_format = prop.validation_format;

            if let Some(validation_format) = maybe_validation_format {
                let args = json!({
                    "value": value,
                    "validation_format": validation_format,
                });

                let resolver = match FuncBinding::create_and_execute(
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

                        dbg!(&message);

                        let status = if message.is_none() {
                            ValidationStatus::Success
                        } else {
                            ValidationStatus::Failure
                        };

                        Validation {
                            prop_id,
                            attribute_value_id,
                            key: None, // TODO(victor) fix validations for keyed values
                            value: ValidationOutput { status, message },
                        }
                    }
                    Err(err) => {
                        // TODO don't swallow errors that aren't function execution errors
                        dbg!(err);
                        Validation {
                            prop_id,
                            attribute_value_id,
                            key: None,
                            value: ValidationOutput {
                                status: ValidationStatus::Error,
                                message: None,
                            },
                        }
                    }
                };
                return Ok(Some(resolver));
            }
        }

        Ok(None)
    }

    pub async fn list_for_component(
        _ctx: &DalContext,
        _component_id: &ComponentId,
    ) -> ValidationResult<Vec<Validation>> {
        Ok(vec![])
    }
}
