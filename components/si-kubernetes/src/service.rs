use crate::model::{
    KubernetesDeploymentComponent, KubernetesDeploymentEntity, KubernetesDeploymentEntityEvent,
};
use crate::protobuf::{
    kubernetes_server, KubernetesDeploymentComponentGetReply,
    KubernetesDeploymentComponentGetRequest, KubernetesDeploymentComponentListReply,
    KubernetesDeploymentComponentListRequest, KubernetesDeploymentComponentPickReply,
    KubernetesDeploymentComponentPickRequest, KubernetesDeploymentEntityCreateEntityReply,
    KubernetesDeploymentEntityCreateEntityRequest,
    KubernetesDeploymentEntityEventListEntityEventsReply,
    KubernetesDeploymentEntityEventListEntityEventsRequest,
    KubernetesDeploymentEntityGetEntityReply, KubernetesDeploymentEntityGetEntityRequest,
    KubernetesDeploymentEntityKubernetesObjectEditReply,
    KubernetesDeploymentEntityKubernetesObjectEditRequest,
    KubernetesDeploymentEntityKubernetesObjectYamlEditReply,
    KubernetesDeploymentEntityKubernetesObjectYamlEditRequest,
    KubernetesDeploymentEntityListEntitiesReply, KubernetesDeploymentEntityListEntitiesRequest,
    KubernetesDeploymentEntitySyncActionReply, KubernetesDeploymentEntitySyncActionRequest,
};
use si_cea::service::prelude::*;

pub type Service = CeaService;

type Component = KubernetesDeploymentComponent;
type Entity = KubernetesDeploymentEntity;
type EntityEvent = KubernetesDeploymentEntityEvent;

#[tonic::async_trait]
impl kubernetes_server::Kubernetes for Service {
    async fn kubernetes_deployment_component_get(
        &self,
        request: TonicRequest<KubernetesDeploymentComponentGetRequest>,
    ) -> TonicResult<KubernetesDeploymentComponentGetReply> {
        const ENDPOINT: &str = "kubernetes_deployment_component_get";
        let db = &self.db;

        async {
            debug!(?request);
            authnz(db, &request, ENDPOINT).await?;

            let component_id = request
                .into_inner()
                .component_id
                .ok_or(CeaError::InvalidComponentGetRequestMissingId)?;

            let output = Component::get(db, &component_id).await?;
            Ok(tonic::Response::new(
                KubernetesDeploymentComponentGetReply {
                    component: Some(output),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_component_list(
        &self,
        request: TonicRequest<KubernetesDeploymentComponentListRequest>,
    ) -> TonicResult<KubernetesDeploymentComponentListReply> {
        const ENDPOINT: &str = "kubernetes_deployment_component_list";
        let db = &self.db;

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut list_request = request.into_inner();

            list_request.default_scope_by_tenant_id(&auth);

            let output = Component::list(db, &list_request).await?;
            Ok(tonic::Response::new(output.into()))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_component_pick(
        &self,
        request: TonicRequest<KubernetesDeploymentComponentPickRequest>,
    ) -> TonicResult<KubernetesDeploymentComponentPickReply> {
        const ENDPOINT: &str = "kubernetes_deployment_component_pick";
        let db = &self.db;

        async {
            debug!(?request);
            authnz(db, &request, ENDPOINT).await?;

            let constraints = request
                .into_inner()
                .constraints
                .ok_or(CeaError::InvalidComponentPickRequestMissingConstraints)?;

            let output = Component::pick(db, &constraints).await?;
            Ok(tonic::Response::new(
                KubernetesDeploymentComponentPickReply {
                    implicit_constraints: Some(output.0),
                    component: Some(output.1),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_create_entity(
        &self,
        request: TonicRequest<KubernetesDeploymentEntityCreateEntityRequest>,
    ) -> TonicResult<KubernetesDeploymentEntityCreateEntityReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_create_entity";
        let db = &self.db;
        let agent = &self.agent;

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let inner = request.into_inner();
            let name = inner
                .name
                .ok_or(CeaError::InvalidEntityCreateRequestMissingField("name"))?;
            let display_name =
                inner
                    .display_name
                    .ok_or(CeaError::InvalidEntityCreateRequestMissingField(
                        "display_name",
                    ))?;
            let description =
                inner
                    .description
                    .ok_or(CeaError::InvalidEntityCreateRequestMissingField(
                        "description",
                    ))?;
            let display_type_name = Component::display_type_name().to_string();
            let properties = inner.properties.unwrap_or_default();
            let constraints = inner.constraints.unwrap_or_default();
            let workspace_id =
                inner
                    .workspace_id
                    .ok_or(CeaError::InvalidEntityCreateRequestMissingField(
                        "workspace_id",
                    ))?;

            let workspace = db.get(&workspace_id).await?;
            let (implicit_constraints, component) = Component::pick(db, &constraints).await?;

            let entity = Entity::create(
                db,
                name,
                display_name,
                description,
                display_type_name,
                properties,
                constraints,
                component,
                implicit_constraints,
                workspace,
            )
            .await?;
            let entity_event = EntityEvent::create(db, auth.user_id(), "create", &entity).await?;
            agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                KubernetesDeploymentEntityCreateEntityReply {
                    entity: Some(entity),
                    entity_event: Some(entity_event),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_list_entities(
        &self,
        request: TonicRequest<KubernetesDeploymentEntityListEntitiesRequest>,
    ) -> TonicResult<KubernetesDeploymentEntityListEntitiesReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_list_entities";
        let db = &self.db;

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut list_request = request.into_inner();

            list_request.default_scope_by_tenant_id(&auth);

            let output = Entity::list(db, &list_request).await?;
            Ok(tonic::Response::new(output.into()))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_get_entity(
        &self,
        request: TonicRequest<KubernetesDeploymentEntityGetEntityRequest>,
    ) -> TonicResult<KubernetesDeploymentEntityGetEntityReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_get_entity";
        let db = &self.db;

        async {
            debug!(?request);
            authnz(db, &request, ENDPOINT).await?;

            let entity_id = request
                .into_inner()
                .entity_id
                .ok_or(CeaError::InvalidEntityGetRequestMissingId)?;

            let output = Entity::get(db, &entity_id).await?;
            Ok(tonic::Response::new(
                KubernetesDeploymentEntityGetEntityReply {
                    entity: Some(output),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_sync_action(
        &self,
        request: TonicRequest<KubernetesDeploymentEntitySyncActionRequest>,
    ) -> TonicResult<KubernetesDeploymentEntitySyncActionReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_sync_action";
        let db = &self.db;
        let agent = &self.agent;

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let entity_id = request
                .into_inner()
                .entity_id
                .ok_or(CeaError::InvalidEntityGetRequestMissingId)?;

            let entity = Entity::get(db, &entity_id).await?;
            let entity_event = EntityEvent::create(db, auth.user_id(), ENDPOINT, &entity).await?;
            agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                KubernetesDeploymentEntitySyncActionReply {
                    entity_event: Some(entity_event),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_kubernetes_object_edit(
        &self,
        request: TonicRequest<KubernetesDeploymentEntityKubernetesObjectEditRequest>,
    ) -> TonicResult<KubernetesDeploymentEntityKubernetesObjectEditReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_kubernetes_object_edit";
        let db = &self.db;
        let agent = &self.agent;
        let inner_property =
            |inner: KubernetesDeploymentEntityKubernetesObjectEditRequest| inner.property;
        let edit_property = |entity: &mut Entity, property| entity.edit_kubernetes_object(property);

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut inner = request.into_inner();
            let entity_id = inner
                .entity_id
                .take()
                .ok_or(CeaError::InvalidEntityEditRequestMissingId)?;
            let property =
                inner_property(inner).ok_or(CeaError::InvalidEntityEditRequestMissingProperty)?;

            let mut entity = Entity::get(db, &entity_id).await?;
            let previous_entity = entity.clone();
            entity.set_state_transition();
            edit_property(&mut entity, property)?;
            entity.save(db).await?;

            let entity_event = EntityEvent::create_with_previous_entity(
                db,
                auth.user_id(),
                ENDPOINT,
                &entity,
                previous_entity,
            )
            .await?;
            agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                KubernetesDeploymentEntityKubernetesObjectEditReply {
                    entity_event: Some(entity_event),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_kubernetes_object_yaml_edit(
        &self,
        request: TonicRequest<KubernetesDeploymentEntityKubernetesObjectYamlEditRequest>,
    ) -> TonicResult<KubernetesDeploymentEntityKubernetesObjectYamlEditReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_kubernetes_object_yaml_edit";
        let db = &self.db;
        let agent = &self.agent;
        let inner_property =
            |inner: KubernetesDeploymentEntityKubernetesObjectYamlEditRequest| inner.property;
        let edit_property =
            |entity: &mut Entity, property| entity.edit_kubernetes_object_yaml(property);

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut inner = request.into_inner();
            let entity_id = inner
                .entity_id
                .take()
                .ok_or(CeaError::InvalidEntityEditRequestMissingId)?;
            let property =
                inner_property(inner).ok_or(CeaError::InvalidEntityEditRequestMissingProperty)?;

            let mut entity = Entity::get(db, &entity_id).await?;
            let previous_entity = entity.clone();
            entity.set_state_transition();
            edit_property(&mut entity, property)?;
            entity.save(db).await?;

            let entity_event = EntityEvent::create_with_previous_entity(
                db,
                auth.user_id(),
                ENDPOINT,
                &entity,
                previous_entity,
            )
            .await?;
            agent.dispatch(&entity_event).await?;

            Ok(tonic::Response::new(
                KubernetesDeploymentEntityKubernetesObjectYamlEditReply {
                    entity_event: Some(entity_event),
                },
            ))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }

    async fn kubernetes_deployment_entity_event_list_entity_events(
        &self,
        request: TonicRequest<KubernetesDeploymentEntityEventListEntityEventsRequest>,
    ) -> TonicResult<KubernetesDeploymentEntityEventListEntityEventsReply> {
        const ENDPOINT: &str = "kubernetes_deployment_entity_event_list_entity_events";
        let db = &self.db;

        async {
            debug!(?request);
            let auth = authnz(db, &request, ENDPOINT).await?;

            let mut list_request = request.into_inner();

            list_request.default_scope_by_tenant_id(&auth);

            let output = EntityEvent::list(db, &list_request).await?;
            Ok(tonic::Response::new(output.into()))
        }
        .instrument(debug_span!(ENDPOINT))
        .await
    }
}
