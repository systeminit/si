use si_data::Db;

use crate::agent::client::MqttAsyncClientInternal;
use crate::entity_event::EntityEvent;
use crate::error::{CeaError, Result};

use std::collections::HashMap;
use std::sync::Arc;

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
    ) -> Result<()> {
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
    async fn setup(&mut self, db: &Db) -> Result<()>;
    async fn dispatch(
        &self,
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EE,
        integration_service_id: String,
        action_name: String,
    ) -> Result<()>;
    fn keys(&self) -> std::collections::hash_map::Keys<(String, String), String>;
}

#[macro_export]
macro_rules! gen_dispatcher {
    (self_ident: $self_ident:ident) => {
        #[derive(Debug, Clone)]
        pub struct Dispatcher(pub si_cea::AgentDispatch);

        impl Deref for Dispatcher {
            type Target = si_cea::AgentDispatch;

            fn deref(&$self_ident) -> &Self::Target {
                &$self_ident.0
            }
        }

        impl DerefMut for Dispatcher {
            fn deref_mut(&mut $self_ident) -> &mut Self::Target {
                &mut $self_ident.0
            }
        }
    }
}

#[macro_export]
macro_rules! gen_dispatch_keys {
    ($self_ident:ident) => {
        fn keys(&$self_ident) -> std::collections::hash_map::Keys<(String, String), String> {
            $self_ident.dispatch_table.keys()
        }
    }
}

#[macro_export]
macro_rules! gen_dispatch_setup {
    (
        $self_ident:ident,
        $db_ident:ident,
        $({
            integration_name: $integration_name:tt,
            integration_service_name: $integration_service_name:tt,
            dispatch[
                $(($action_name:tt, $dispatch_to:path)),*
            ]
        }),*
    ) => {
        $(
        {
            let integration: si_account::Integration = $db_ident
                .lookup_by_natural_key(format!("global:integration:{}", $integration_name))
                .await?;
            let integration_service_lookup_id = format!(
                "global:{}:integration_service:{}",
                integration.id,
                $integration_service_name
            );
            let integration_service: si_account::IntegrationService = $db_ident
                .lookup_by_natural_key(integration_service_lookup_id)
                .await?;
            $(
                $self_ident.dispatch_to(integration_service.id.clone(), $action_name, stringify!($dispatch_to))?;
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
