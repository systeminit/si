use crate::model::component::Component;
use crate::model::entity::Entity;
use crate::model::entity_event::EntityEvent;
use crate::protobuf::kubernetes_deployment::{
    kubernetes_deployment_server, Constraints, CreateEntityReply, CreateEntityRequest,
    EditKubernetesObjectActionReply, EditKubernetesObjectActionRequest,
    EditKubernetesObjectYamlActionReply, EditKubernetesObjectYamlActionRequest, GetComponentReply,
    GetComponentRequest, GetEntityReply, GetEntityRequest, ListComponentsReply,
    ListComponentsRequest, ListEntitiesReply, ListEntitiesRequest, ListEntityEventsReply,
    ListEntityEventsRequest, PickComponentReply, PickComponentRequest, SyncActionReply,
    SyncActionRequest,
};
use si_cea::service::prelude::*;

pub type Service = CeaService;

#[tonic::async_trait]
impl kubernetes_deployment_server::KubernetesDeployment for Service {
    async fn get_component(
        &self,
        request: TonicRequest<GetComponentRequest>,
    ) -> TonicResult<GetComponentReply> {
        component_get!(Component, "get_component", request, &self.db)
    }

    async fn list_components(
        &self,
        request: TonicRequest<ListComponentsRequest>,
    ) -> TonicResult<ListComponentsReply> {
        component_list!(Component, "list_components", request, &self.db)
    }

    async fn pick_component(
        &self,
        request: TonicRequest<PickComponentRequest>,
    ) -> TonicResult<PickComponentReply> {
        component_pick!(Component, "pick_component", request, &self.db)
    }

    async fn create_entity(
        &self,
        request: TonicRequest<CreateEntityRequest>,
    ) -> TonicResult<CreateEntityReply> {
        entity_create!(
            Component,
            Entity,
            EntityEvent,
            "create_entity",
            request,
            &self.db,
            &self.agent,
        )
    }

    async fn list_entities(
        &self,
        request: TonicRequest<ListEntitiesRequest>,
    ) -> TonicResult<ListEntitiesReply> {
        entity_list!(Entity, "list_entities", request, &self.db)
    }

    async fn get_entity(
        &self,
        request: TonicRequest<GetEntityRequest>,
    ) -> TonicResult<GetEntityReply> {
        entity_get!(Entity, "get_entity", request, &self.db)
    }

    async fn sync_action(
        &self,
        request: TonicRequest<SyncActionRequest>,
    ) -> TonicResult<SyncActionReply> {
        sync!(Entity, EntityEvent, "sync", request, &self.db, &self.agent)
    }

    async fn edit_kubernetes_object_action(
        &self,
        request: TonicRequest<EditKubernetesObjectActionRequest>,
    ) -> TonicResult<EditKubernetesObjectActionReply> {
        edit!(
            Entity,
            EntityEvent,
            "edit_kubernetes_object",
            request,
            &self.db,
            &self.agent,
            |inner: EditKubernetesObjectActionRequest| inner.kubernetes_object,
            |entity: &mut Entity, property| entity.edit_kubernetes_object(property),
        )
    }

    async fn edit_kubernetes_object_yaml_action(
        &self,
        request: TonicRequest<EditKubernetesObjectYamlActionRequest>,
    ) -> TonicResult<EditKubernetesObjectYamlActionReply> {
        edit!(
            Entity,
            EntityEvent,
            "edit_kubernetes_object_yaml",
            request,
            &self.db,
            &self.agent,
            |inner: EditKubernetesObjectYamlActionRequest| inner.kubernetes_object_yaml,
            |entity: &mut Entity, property| entity.edit_kubernetes_object_yaml(property),
        )
    }

    async fn list_entity_events(
        &self,
        request: TonicRequest<ListEntityEventsRequest>,
    ) -> TonicResult<ListEntityEventsReply> {
        entity_event_list!(EntityEvent, "list_entity_events", request, &self.db)
    }
}

impl From<Component> for GetComponentReply {
    fn from(component: Component) -> Self {
        Self {
            component: Some(component),
        }
    }
}

impl From<(Constraints, Component)> for PickComponentReply {
    fn from((implicit_constraints, component): (Constraints, Component)) -> Self {
        Self {
            implicit_constraints: Some(implicit_constraints),
            component: Some(component),
        }
    }
}

impl From<(Entity, EntityEvent)> for CreateEntityReply {
    fn from((entity, entity_event): (Entity, EntityEvent)) -> Self {
        Self {
            entity: Some(entity),
            entity_event: Some(entity_event),
        }
    }
}

impl From<Entity> for GetEntityReply {
    fn from(entity: Entity) -> Self {
        Self {
            entity: Some(entity),
        }
    }
}

impl From<EntityEvent> for SyncActionReply {
    fn from(entity_event: EntityEvent) -> Self {
        Self {
            entity_event: Some(entity_event),
        }
    }
}

impl From<EntityEvent> for EditKubernetesObjectActionReply {
    fn from(entity_event: EntityEvent) -> Self {
        Self {
            entity_event: Some(entity_event),
        }
    }
}

impl From<EntityEvent> for EditKubernetesObjectYamlActionReply {
    fn from(entity_event: EntityEvent) -> Self {
        Self {
            entity_event: Some(entity_event),
        }
    }
}
