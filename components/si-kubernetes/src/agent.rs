use si_cea::{CeaResult, MqttAsyncClientInternal};
use si_data::Db;

use si_cea::agent::dispatch::prelude::*;

type MyEntityEvent = crate::model::EntityEvent;

#[async_trait::async_trait]
pub trait IntegrationServceDispatchV2 {
    type EntityEvent: si_cea::EntityEvent;

    fn actions() -> &'static [&'static str];

    async fn dispatch(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;
}

pub struct IntegrationServceDispatcher<T: IntegrationServceDispatchV2> {
    _phantom: std::marker::PhantomData<T>,
}

// pub struct DispatcherV2<EE> {
//     integrations: std::collections::HashMap<String, Box<dyn IntegrationServceDispatchV2<EE>>>,
// }

// ------------------

#[derive(Debug, Clone)]
pub struct Dispatcher(pub AgentDispatch);

impl std::ops::Deref for Dispatcher {
    type Target = AgentDispatch;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Dispatcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Dispatcher {}

#[async_trait::async_trait]
impl Dispatch<MyEntityEvent> for Dispatcher {
    fn keys(&self) -> Vec<(String, String)> {
        vec![]
    }

    // fn keys(&self) -> std::collections::hash_map::Keys<(String, String), String> {
    //     self.dispatch_table.keys()
    // }

    // async fn setup(&mut self, db: &Db) -> CeaResult<()> {
    //     gen_dispatch_setup!(self, db, {
    //         integration_name: "aws",
    //         integration_service_name: "eks_kubernetes",
    //         dispatch[
    //             ("create", aws::create),
    //             ("sync", aws::sync),
    //             ("edit", aws::edit)
    //         ]
    //     });

    //     let table: std::collections::HashMap<(String, String), DispatchFunction>;

    //     Ok(())
    // }

    async fn dispatch(
        &self,
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut MyEntityEvent,
    ) -> CeaResult<()> {
        // gen_dispatch!(
        //     self,
        //     mqtt_client,
        //     entity_event,
        //     integration_service_id,
        //     action_name,
        //     dispatch[
        //         aws::create,
        //         aws::sync,
        //         aws::edit
        //     ]
        // );

        let integration_service_id = entity_event.integration_service_id();
        let action_name = entity_event.action_name();

        // match (integration_service_id, action_name) {
        //     ("eks_kubernetes", "create") => {
        //         aws::AwsDispatchFunctionsImpl::create(mqtt_client, entity_event).await?
        //     }
        //     ("eks_kubernetes", "sync") => {
        //         aws::AwsDispatchFunctionsImpl::sync(mqtt_client, entity_event).await?
        //     }
        //     ("eks_kubernetes", "edit_kubernetes_object") => {
        //         aws::AwsDispatchFunctionsImpl::edit_kubernetes_object(mqtt_client, entity_event)
        //             .await?
        //     }
        //     ("eks_kubernetes", "edit_kubernetes_object_yaml") => {
        //         aws::AwsDispatchFunctionsImpl::edit_kubernetes_object_yaml(
        //             mqtt_client,
        //             entity_event,
        //         )
        //         .await?
        //     }
        // }

        Ok(())
    }
}

#[async_trait::async_trait]
pub trait IntegrationServceDispatch<EE: si_cea::EntityEvent> {
    fn actions() -> &'static [&'static str];

    async fn dispatch(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EE,
    ) -> CeaResult<()>;
}

#[async_trait::async_trait]
pub trait AwsDispatchFunctions {
    type EntityEvent: si_cea::EntityEvent;

    async fn create(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;

    async fn sync(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;

    async fn edit_kubernetes_object(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;

    async fn edit_kubernetes_object_yaml(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()>;
}

pub struct AwsDispatcher<T: AwsDispatchFunctions> {
    _phantom: std::marker::PhantomData<T>,
}

#[async_trait::async_trait]
impl<T: AwsDispatchFunctions + 'static> IntegrationServceDispatch<T::EntityEvent>
    for AwsDispatcher<T>
{
    fn actions() -> &'static [&'static str] {
        &[
            "create",
            "sync",
            "edit_kubernetes_object",
            "edit_kubernetes_object_yaml",
        ]
    }

    async fn dispatch(
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut T::EntityEvent,
    ) -> CeaResult<()> {
        let action = entity_event.action_name();

        match action {
            "create" => T::create(mqtt_client, entity_event).await,
            "sync" => T::sync(mqtt_client, entity_event).await,
            "edit_kubernetes_object" => T::edit_kubernetes_object(mqtt_client, entity_event).await,
            "edit_kubernetes_object_yaml" => {
                T::edit_kubernetes_object_yaml(mqtt_client, entity_event).await
            }
            invalid => panic!("TODO: fix for invalid action: {}", invalid),
        }
    }
}

mod aws {
    use crate::model::entity::Entity;
    use crate::model::entity_event::EntityEvent;
    use si_cea::{CeaError, CeaResult, EntityEvent as _, MqttAsyncClientInternal};
    use tracing::debug_span;
    use tracing_futures::Instrument as _;

    type MyEntityEvent = crate::model::entity_event::EntityEvent;

    pub struct AwsDispatchFunctionsImpl;

    #[async_trait::async_trait]
    impl super::AwsDispatchFunctions for AwsDispatchFunctionsImpl {
        type EntityEvent = MyEntityEvent;

        async fn create(
            _mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            async {
                entity_event.log("Kubernetes like a motherfucker\n");
                entity_event.log(format!("{:?}", entity_event.input_entity()));
                entity_event.init_output_entity();
                Ok(())
            }
            .instrument(debug_span!("create"))
            .await
        }

        async fn sync(
            _mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            async {
                entity_event.log("Synchronizing like we just dont care\n");
                entity_event.init_output_entity();
                Ok(())
            }
            .instrument(debug_span!("sync"))
            .await
        }

        async fn edit_kubernetes_object(
            _mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            async {
                entity_event.log("Editing kubernetes_object like we just dont care\n");
                entity_event.init_output_entity();
                Ok(())
            }
            .instrument(debug_span!("edit_kubernetes_object"))
            .await
        }

        async fn edit_kubernetes_object_yaml(
            _mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            async {
                entity_event.log("Editing kubernetes_object_yaml like we just dont care\n");
                entity_event.init_output_entity();
                Ok(())
            }
            .instrument(debug_span!("edit_kubernetes_object_yaml"))
            .await
        }
    }
}
