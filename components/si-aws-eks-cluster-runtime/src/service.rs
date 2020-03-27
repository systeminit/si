use crate::model::component::Component;
use crate::model::entity::{Entity, EntityEvent};
use crate::protobuf::{
    aws_eks_cluster_runtime_server, AddNodegroupReply, AddNodegroupRequest, CreateEntityReply,
    CreateEntityRequest, GetComponentReply, GetComponentRequest, GetEntityReply, GetEntityRequest,
    ImplicitConstraint, ListComponentsReply, ListComponentsRequest, ListEntitiesReply,
    ListEntitiesRequest, ListEntityEventsReply, ListEntityEventsRequest, PickComponentReply,
    PickComponentRequest, SyncEntityReply, SyncEntityRequest,
};
use si_cea::service::prelude::*;

pub type Service = CeaService;

#[tonic::async_trait]
impl aws_eks_cluster_runtime_server::AwsEksClusterRuntime for Service {
    async fn sync_entity(
        &self,
        request: TonicRequest<SyncEntityRequest>,
    ) -> TonicResult<SyncEntityReply> {
        gen_service_action!(self, request, "sync_entity", "sync", SyncEntityReply)
    }

    async fn list_entity_events(
        &self,
        mut request: TonicRequest<ListEntityEventsRequest>,
    ) -> TonicResult<ListEntityEventsReply> {
        gen_service_list!(self, request, "list_entity_events", EntityEvent)
    }

    async fn create_entity(
        &self,
        request: TonicRequest<CreateEntityRequest>,
    ) -> TonicResult<CreateEntityReply> {
        gen_service_create_entity!(self, request, "create_entity", CreateEntityReply)
    }

    async fn pick_component(
        &self,
        request: TonicRequest<PickComponentRequest>,
    ) -> TonicResult<PickComponentReply> {
        gen_service_pick_component!(self, request, "pick_component", PickComponentReply)
    }

    async fn list_components(
        &self,
        mut request: TonicRequest<ListComponentsRequest>,
    ) -> TonicResult<ListComponentsReply> {
        gen_service_list!(self, request, "list_components", Component)
    }

    async fn get_component(
        &self,
        request: TonicRequest<GetComponentRequest>,
    ) -> TonicResult<GetComponentReply> {
        gen_service_get!(
            self,
            request,
            "get_component",
            Component,
            component_id,
            GetComponentReply,
            component
        )
    }

    async fn list_entities(
        &self,
        mut request: TonicRequest<ListEntitiesRequest>,
    ) -> TonicResult<ListEntitiesReply> {
        gen_service_list!(self, request, "list_entities", Entity)
    }

    async fn get_entity(
        &self,
        request: TonicRequest<GetEntityRequest>,
    ) -> TonicResult<GetEntityReply> {
        gen_service_get!(
            self,
            request,
            "get_entity",
            Entity,
            entity_id,
            GetEntityReply,
            entity
        )
    }

    async fn add_nodegroup(
        &self,
        request: TonicRequest<AddNodegroupRequest>,
    ) -> TonicResult<AddNodegroupReply> {
        gen_service_action!(
            self,
            request,
            "add_nodegroup",
            "add_nodegroup",
            AddNodegroupReply
        )
    }
}
