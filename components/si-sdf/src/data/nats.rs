use crate::models::{key_pair, PublicKey};
use ::nats::asynk::{self, Connection};
use serde::Serialize;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::Mutex;

#[derive(Error, Debug)]
pub enum NatsTxnError {
    #[error("serde error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("missing type name on object")]
    MissingTypeName,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type NatsTxnResult<T> = Result<T, NatsTxnError>;

#[derive(Clone, Debug)]
pub struct NatsConn {
    conn: Connection,
}

impl NatsConn {
    pub async fn new(settings: &si_settings::Nats) -> NatsTxnResult<Self> {
        let conn = asynk::connect(&settings.url).await?;

        Ok(Self { conn })
    }

    pub fn transaction(&self) -> NatsTxn {
        NatsTxn::new(self.conn.clone())
    }

    pub async fn subscribe(&self, subject: &str) -> std::io::Result<::nats::asynk::Subscription> {
        self.conn.subscribe(subject).await
    }
}

#[derive(Debug, Clone)]
pub struct NatsTxn {
    connection: Connection,
    object_list: Arc<Mutex<Vec<serde_json::Value>>>,
}

impl NatsTxn {
    fn new(connection: Connection) -> Self {
        NatsTxn {
            connection,
            object_list: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn publish<T: Serialize + std::fmt::Debug>(&self, object: &T) -> NatsTxnResult<()> {
        let json: serde_json::Value = serde_json::to_value(object)?;
        let mut object_list = self.object_list.lock().await;
        object_list.push(json);
        Ok(())
    }

    pub async fn delete<T: Serialize + std::fmt::Debug>(&self, object: &T) -> NatsTxnResult<()> {
        let json: serde_json::Value = serde_json::to_value(object)?;
        let mut object_list = self.object_list.lock().await;
        object_list.push(serde_json::json![{ "deleted": json }]);
        Ok(())
    }

    pub async fn commit(self) -> NatsTxnResult<()> {
        let mut object_list = self.object_list.lock().await;
        for model_json in object_list.iter_mut() {
            let mut model_body: serde_json::Value = model_json.clone();
            if let Some(type_name) = model_json["siStorable"]["typeName"].as_str() {
                match type_name {
                    "userPassword" | "jwtKeyPrivate" | "jwtKeyPublic" => return Ok(()),
                    "keyPair" => {
                        let key_pair: key_pair::KeyPair =
                            serde_json::from_value(model_json.clone())?;
                        *model_json = serde_json::to_value(PublicKey::from(key_pair))?;
                    }
                    _ => (),
                }
            } else if model_json["deleted"].is_object() {
                model_body = model_json["deleted"].clone();
            } else {
                dbg!(model_json);
                return Err(NatsTxnError::MissingTypeName);
            }
            let mut subject_array: Vec<String> = Vec::new();
            if let Some(tenant_ids_values) = model_body["siStorable"]["tenantIds"].as_array() {
                for tenant_id_value in tenant_ids_values.iter() {
                    let tenant_id = String::from(tenant_id_value.as_str().unwrap());
                    subject_array.push(tenant_id);
                }
            } else {
                match model_body["siStorable"]["billingAccountId"].as_str() {
                    Some(billing_account_id) => subject_array.push(billing_account_id.into()),
                    None => return Ok(()),
                }
            }
            if subject_array.len() != 0 {
                let subject: String = subject_array.join(".");
                self.connection
                    .publish(&subject, model_json.to_string())
                    .await?;
            } else {
                dbg!(&model_json);
                dbg!("tried to publish a model that has no tenancy!");
            }
        }
        Ok(())
    }
}
