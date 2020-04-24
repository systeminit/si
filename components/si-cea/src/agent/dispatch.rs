use crate::agent::client::MqttAsyncClientInternal;
use crate::entity_event::EntityEvent;
use crate::error::{CeaError, CeaResult};
use std::collections::HashMap;
use std::sync::Arc;

pub mod prelude {
    pub use super::{AgentDispatch, Dispatch};
    pub use crate::entity_event::EntityEvent as _;
}

type IntegrationServiceId = String;
type ActionName = String;
type FunctionName = String;

#[derive(Debug, Clone)]
pub struct AgentDispatch {
    pub dispatch_table: Arc<HashMap<(IntegrationServiceId, ActionName), FunctionName>>,
}

impl AgentDispatch {
    pub fn new() -> AgentDispatch {
        AgentDispatch {
            dispatch_table: Arc::new(HashMap::new()),
        }
    }

    pub fn keys(&self) -> std::collections::hash_map::Keys<(String, String), String> {
        self.dispatch_table.keys()
    }

    pub fn lookup(&self, integration_service_id: String, action_name: String) -> Option<&str> {
        self.dispatch_table
            .get(&(integration_service_id, action_name))
            .map(|s| &s[..])
    }

    pub fn dispatch_to(
        &mut self,
        integration_service_id: impl Into<IntegrationServiceId>,
        action_name: impl Into<ActionName>,
        function_name: impl Into<FunctionName>,
    ) -> CeaResult<()> {
        let dispatch_table =
            Arc::get_mut(&mut self.dispatch_table).ok_or(CeaError::ExternalRequest)?;
        dispatch_table.insert(
            (integration_service_id.into(), action_name.into()),
            function_name.into(),
        );
        Ok(())
    }
}

#[async_trait::async_trait]
pub trait Dispatch<EE: EntityEvent> {
    // async fn setup(&mut self, db: &Db) -> CeaResult<()>;
    async fn dispatch(
        &self,
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EE,
    ) -> CeaResult<()>;
    fn keys(&self) -> Vec<(String, String)>;
}

#[macro_export]
macro_rules! gen_dispatch_setup {
    (
        $self_ident:ident,
        $db:ident,
        $({
            integration_name: $integration_name:tt,
            integration_service_name: $integration_service_name:tt,
            dispatch[
                $(($action_name:tt, $dispatch_to:path)),*
            ]
        }),*
    ) => {
        let db = $db;
        $(
        {
            let integration_name = $integration_name;
            let integration_service_name = $integration_service_name;

            let integration: si_account::Integration = db
                .lookup_by_natural_key(format!("global:integration:{}", integration_name))
                .await?;
            let integration_service_lookup_id = format!(
                "global:{}:integration_service:{}",
                integration.id,
                integration_service_name
            );
            let integration_service: si_account::IntegrationService = db
                .lookup_by_natural_key(integration_service_lookup_id)
                .await?;
            $(
                let action_name = $action_name;
                $self_ident.dispatch_to(integration_service.id.clone(), action_name, stringify!($dispatch_to))?;
            )*
        }
        )*
    }
}

#[macro_export]
macro_rules! gen_dispatch {
    ( $self_ident:ident,
      $mqtt_client:ident,
      $entity_event:ident,
      $integration_service_id:ident,
      $action_name:ident,
      dispatch[
        $($dispatch_to:path),*
      ]
    ) => {
        match $self_ident.lookup($integration_service_id.clone(), $action_name.clone()) {
            $(
                Some(stringify!($dispatch_to)) => $dispatch_to(&$mqtt_client, $entity_event).await?,
            )*
                Some(_) => {
                    return Err(si_cea::error::CeaError::DispatchFunctionMissing(
                            $integration_service_id,
                            $action_name,
                    ))
                },
            None => {
                return Err(si_cea::error::CeaError::DispatchFunctionMissing(
                        $integration_service_id,
                        $action_name,
                ))
            }
        }
    }
}
