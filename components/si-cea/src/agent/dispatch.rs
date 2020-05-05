use crate::{CeaError, CeaResult, EntityEvent, MqttClient};
use async_trait::async_trait;
use dyn_clone::DynClone;
use si_data::{DataError, Db};
use std::collections::HashMap;

pub mod codegen_prelude {
    pub use super::{Dispatch, IntegrationActions, IntegrationAndServiceName, IntegrationDispatch};
    pub use crate::agent::mqtt::MqttClient;
    pub use crate::entity_event::EntityEvent;
    pub use crate::error::{CeaError, CeaResult, TonicResult};
    pub use async_trait::async_trait;
    pub use std::marker::PhantomData;
}

pub mod prelude {
    pub use super::{IntegrationActions, IntegrationAndServiceName};
    pub use crate::agent::mqtt::MqttClient;
    pub use crate::entity_event::EntityEvent as _;
    pub use crate::error::CeaResult;
    pub use async_trait::async_trait;
    pub use tracing::debug_span;
    pub use tracing_futures::Instrument as _;
}

#[async_trait]
pub trait Dispatch {
    type EntityEvent: EntityEvent;

    async fn dispatch(
        &self,
        mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;
}

pub trait IntegrationActions {
    fn integration_actions(&self) -> &'static [&'static str];
}

pub trait IntegrationDispatch: Dispatch + IntegrationActions + Sync + Send + DynClone {}

impl<T: EntityEvent> Clone for Box<dyn IntegrationDispatch<EntityEvent = T>> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

pub trait IntegrationAndServiceName {
    fn integration_name() -> &'static str;
    fn integration_service_name() -> &'static str;

    fn type_integration_name(&self) -> &'static str {
        Self::integration_name()
    }
    fn type_integration_service_name(&self) -> &'static str {
        Self::integration_service_name()
    }
}

pub trait SubscribeKeys {
    fn subscribe_keys(&self) -> Vec<SubscribeKey>;
}

pub struct SubscribeKey<'a> {
    integration_service_id: &'a str,
    action_name: &'a str,
}

impl<'a> SubscribeKey<'a> {
    pub fn integration_service_id(&self) -> &str {
        self.integration_service_id
    }

    pub fn action_name(&self) -> &str {
        self.action_name
    }
}

#[derive(Default, Clone)]
pub struct Dispatcher<T: EntityEvent> {
    dispatchers: HashMap<String, Box<dyn IntegrationDispatch<EntityEvent = T>>>,
}

impl<T: EntityEvent> Dispatcher<T> {
    pub async fn add(
        &mut self,
        db: &Db,
        dispatcher: impl IntegrationDispatch<EntityEvent = T> + IntegrationAndServiceName + 'static,
    ) -> CeaResult<()> {
        let integration_name = dispatcher.type_integration_name();
        let integration_service_name = dispatcher.type_integration_service_name();

        let id = integration_service_id_for(db, integration_name, integration_service_name).await?;
        self.dispatchers.insert(id, Box::new(dispatcher));

        Ok(())
    }
}

#[async_trait]
impl<T: EntityEvent> Dispatch for Dispatcher<T> {
    type EntityEvent = T;

    async fn dispatch(
        &self,
        mqtt_client: &MqttClient,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        match self.dispatchers.get(entity_event.integration_service_id()?) {
            Some(dispatcher) => dispatcher.dispatch(mqtt_client, entity_event).await,
            None => Err(CeaError::DispatchFunctionMissing(
                entity_event.integration_service_id()?.to_string(),
                entity_event.action_name()?.to_string(),
            )),
        }
    }
}

impl<T: EntityEvent> SubscribeKeys for Dispatcher<T> {
    fn subscribe_keys(&self) -> Vec<SubscribeKey> {
        self.dispatchers
            .iter()
            .flat_map(|(integration_service_id, dispatcher)| {
                dispatcher
                    .integration_actions()
                    .iter()
                    .map(move |action_name| SubscribeKey {
                        integration_service_id,
                        action_name,
                    })
            })
            .collect()
    }
}

async fn integration_service_id_for(
    db: &Db,
    integration_name: impl AsRef<str>,
    integration_service_name: impl AsRef<str>,
) -> CeaResult<String> {
    let integration_name = integration_name.as_ref();
    let integration_service_name = integration_service_name.as_ref();

    let integration: si_account::Integration = db
        .lookup_by_natural_key(format!("global:integration:{}", integration_name))
        .await?;
    let integration_service_lookup_id = format!(
        "global:{}:integration_service:{}",
        integration
            .id
            .ok_or(DataError::RequiredField("id".to_string()))?,
        integration_service_name
    );
    let integration_service: si_account::IntegrationService = db
        .lookup_by_natural_key(integration_service_lookup_id)
        .await?;

    Ok(integration_service
        .id
        .ok_or(DataError::RequiredField("id".to_string()))?)
}
