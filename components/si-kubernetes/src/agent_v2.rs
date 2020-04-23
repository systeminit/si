/// --------------------------
/// Library/common
/// --------------------------
///
///
pub mod lib {
    use async_trait::async_trait;
    use si_cea::{CeaResult, EntityEvent as EntityEventTrait, MqttAsyncClientInternal};
    use si_data::Db;
    use std::collections::HashMap;

    pub mod codegen_prelude {
        pub use super::{Dispatch, IntegrationActions, IntegrationNames};
        pub use async_trait::async_trait;
        pub use si_cea::{CeaResult, EntityEvent as EntityEventTrait, MqttAsyncClientInternal};
        pub use std::marker::PhantomData;
    }

    pub mod prelude {
        pub use super::IntegrationNames;
        pub use async_trait::async_trait;
        pub use si_cea::{CeaResult, EntityEvent as _, MqttAsyncClientInternal};
        pub use tracing::debug_span;
        pub use tracing_futures::Instrument as _;
    }

    #[async_trait]
    pub trait Dispatch {
        type EntityEvent: EntityEventTrait;

        async fn dispatch(
            &self,
            mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()>;
    }

    pub trait IntegrationActions {
        fn actions() -> &'static [&'static str];
    }

    pub trait IntegrationNames {
        fn integration_names() -> (&'static str, &'static str);

        fn type_integration_names(&self) -> (&'static str, &'static str) {
            Self::integration_names()
        }
    }

    #[derive(Default)]
    pub struct Dispatcher<T> {
        dispatchers: HashMap<String, Box<dyn Dispatch<EntityEvent = T> + Sync + Send>>,
    }

    impl<T: EntityEventTrait> Dispatcher<T> {
        pub async fn add(
            &mut self,
            db: &Db,
            dispatcher: impl Dispatch<EntityEvent = T> + IntegrationNames + Sync + Send + 'static,
        ) -> CeaResult<()> {
            let (integration_name, integration_service_name) = dispatcher.type_integration_names();

            let id =
                integration_service_id_for(db, integration_name, integration_service_name).await?;
            self.dispatchers.insert(id, Box::new(dispatcher));

            Ok(())
        }
    }

    #[async_trait]
    impl<T: EntityEventTrait> Dispatch for Dispatcher<T> {
        type EntityEvent = T;

        async fn dispatch(
            &self,
            _mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            todo!("boop")
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
            integration.id, integration_service_name
        );
        let integration_service: si_account::IntegrationService = db
            .lookup_by_natural_key(integration_service_lookup_id)
            .await?;

        Ok(integration_service.id)
    }
}

/// --------------------------
/// Codegen
/// --------------------------
///
///
pub mod codegen {
    use super::lib::codegen_prelude::*;

    ///
    /// AWS
    ///

    pub struct AwsDispatcher<T: AwsDispatchFunctions> {
        _phantom: PhantomData<T>,
    }

    impl<T: AwsDispatchFunctions> AwsDispatcher<T> {
        pub fn new() -> Self {
            Self {
                _phantom: PhantomData::default(),
            }
        }
    }

    impl<T: AwsDispatchFunctions> IntegrationActions for AwsDispatcher<T> {
        fn actions() -> &'static [&'static str] {
            &[
                "create",
                "sync",
                "edit_kubernetes_object",
                "edit_kubernetes_object_yaml",
            ]
        }
    }

    impl<T: AwsDispatchFunctions> IntegrationNames for AwsDispatcher<T> {
        fn integration_names() -> (&'static str, &'static str) {
            T::integration_names()
        }
    }

    #[async_trait]
    impl<T: AwsDispatchFunctions + Sync> Dispatch for AwsDispatcher<T> {
        type EntityEvent = T::EntityEvent;

        async fn dispatch(
            &self,
            mc: &MqttAsyncClientInternal,
            ee: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            match ee.action_name() {
                "create" => T::create(mc, ee).await,
                "sync" => T::sync(mc, ee).await,
                "edit_kubernetes_object" => T::edit_kubernetes_object(mc, ee).await,
                "edit_kubernetes_object_yaml" => T::edit_kubernetes_object_yaml(mc, ee).await,
                invalid => panic!("TODO: fix for invalid action: {}", invalid),
            }
        }
    }

    #[async_trait]
    pub trait AwsDispatchFunctions: IntegrationNames {
        type EntityEvent: EntityEventTrait + Send;

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

    ///
    /// GCP
    ///

    pub struct GcpDispatcher<T: GcpDispatchFunctions> {
        _phantom: PhantomData<T>,
    }

    impl<T: GcpDispatchFunctions> GcpDispatcher<T> {
        pub fn new() -> Self {
            Self {
                _phantom: PhantomData::default(),
            }
        }
    }

    impl<T: GcpDispatchFunctions> IntegrationActions for GcpDispatcher<T> {
        fn actions() -> &'static [&'static str] {
            &["create", "sync", "edit_kubernetes_object_yaml"]
        }
    }

    impl<T: GcpDispatchFunctions> IntegrationNames for GcpDispatcher<T> {
        fn integration_names() -> (&'static str, &'static str) {
            T::integration_names()
        }
    }

    #[async_trait]
    impl<T: GcpDispatchFunctions + Sync> Dispatch for GcpDispatcher<T> {
        type EntityEvent = T::EntityEvent;

        async fn dispatch(
            &self,
            mc: &MqttAsyncClientInternal,
            ee: &mut Self::EntityEvent,
        ) -> CeaResult<()> {
            match ee.action_name() {
                "create" => T::create(mc, ee).await,
                "sync" => T::sync(mc, ee).await,
                "edit_kubernetes_object_yaml" => T::edit_kubernetes_object_yaml(mc, ee).await,
                invalid => panic!("TODO: fix for invalid action: {}", invalid),
            }
        }
    }

    #[async_trait]
    pub trait GcpDispatchFunctions: IntegrationNames {
        type EntityEvent: EntityEventTrait + Send;

        async fn create(
            mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()>;

        async fn sync(
            mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()>;

        async fn edit_kubernetes_object_yaml(
            mqtt_client: &MqttAsyncClientInternal,
            entity_event: &mut Self::EntityEvent,
        ) -> CeaResult<()>;
    }
}

/// --------------------------
/// Crate consumer
/// --------------------------
///
///
pub mod mycrate {
    use super::codegen::{AwsDispatcher, GcpDispatcher};
    use super::lib::Dispatcher;
    use crate::model::EntityEvent;
    use si_cea::CeaResult;

    type MyEntityEvent = EntityEvent;

    // Somewhere in the crate library root
    pub async fn dispatcher(db: &si_data::Db) -> CeaResult<Dispatcher<MyEntityEvent>> {
        let mut dispatcher = Dispatcher::default();

        dispatcher
            .add(db, AwsDispatcher::<aws::AwsDispatchFunctionsImpl>::new())
            .await?;
        dispatcher
            .add(db, GcpDispatcher::<gcp::GcpDispatchFunctionsImpl>::new())
            .await?;

        Ok(dispatcher)
    }

    mod aws {
        use super::super::codegen::AwsDispatchFunctions;
        use super::super::lib::prelude::*;
        use crate::model::EntityEvent;

        type MyEntityEvent = EntityEvent;

        pub struct AwsDispatchFunctionsImpl;

        impl IntegrationNames for AwsDispatchFunctionsImpl {
            fn integration_names() -> (&'static str, &'static str) {
                ("aws", "kubernetes_deployment")
            }
        }

        #[async_trait]
        impl AwsDispatchFunctions for AwsDispatchFunctionsImpl {
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

    mod gcp {
        use super::super::codegen::GcpDispatchFunctions;
        use super::super::lib::prelude::*;
        use crate::model::EntityEvent;

        type MyEntityEvent = EntityEvent;

        pub struct GcpDispatchFunctionsImpl;

        impl IntegrationNames for GcpDispatchFunctionsImpl {
            fn integration_names() -> (&'static str, &'static str) {
                ("gcp", "kubernetes_deployment")
            }
        }

        #[async_trait]
        impl GcpDispatchFunctions for GcpDispatchFunctionsImpl {
            type EntityEvent = MyEntityEvent;

            async fn create(
                _mqtt_client: &MqttAsyncClientInternal,
                _entity_event: &mut Self::EntityEvent,
            ) -> CeaResult<()> {
                todo!("boop")
            }

            async fn sync(
                _mqtt_client: &MqttAsyncClientInternal,
                _entity_event: &mut Self::EntityEvent,
            ) -> CeaResult<()> {
                todo!("boop");
            }

            async fn edit_kubernetes_object_yaml(
                _mqtt_client: &MqttAsyncClientInternal,
                _entity_event: &mut Self::EntityEvent,
            ) -> CeaResult<()> {
                todo!("boop")
            }
        }
    }
}
