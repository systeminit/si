// Auto-generated rust code!
// No-Touchy!

use tonic;
use tracing::{self, debug, info, info_span};
use tracing_futures::Instrument as _;

use si_data;

#[derive(Debug)]
pub struct Service {
    pub db: si_data::Db,
}

impl Service {
    pub fn new(db: si_data::Db) -> Service {
        Service { db }
    }

    pub fn db(&self) -> &si_data::Db {
        &self.db
    }
}

#[tonic::async_trait]
impl crate::protobuf::kubernetes_server::Kubernetes for Service {
    async fn kubernetes_deployment_component_get(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentGetReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_get",
            )
            .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let output =
                crate::protobuf::KubernetesDeploymentComponent::get(&self.db, &request_id).await?;
            info!(?output);
            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentComponentGetReply {
                    object: Some(output),
                },
            ))
        }
        .instrument(info_span!("kubernetes_deployment_component_get"))
        .await
    }

    async fn kubernetes_deployment_component_list(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentListReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_list",
            )
            .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let list_reply =
                crate::protobuf::KubernetesDeploymentComponent::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentComponentListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("kubernetes_deployment_component_list"))
        .await
    }

    async fn kubernetes_deployment_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentComponentPickReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_component_pick",
            )
            .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let output =
                crate::model::KubernetesDeploymentComponent::kubernetes_deployment_component_pick(
                    &self.db, inner,
                )
                .await?;
            info!(?output);
            Ok(tonic::Response::new(output))
        }
        .instrument(info_span!("kubernetes_deployment_component_pick"))
        .await
    }

    async fn kubernetes_deployment_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityCreateReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_create",
            )
            .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let constraints = inner.constraints;
            let properties = inner.properties;
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let workspace_id = inner.workspace_id;
            let output = crate::protobuf::KubernetesDeploymentEntity::create(
                &self.db,
                constraints,
                properties,
                name,
                display_name,
                description,
                workspace_id,
            )
            .await?;
            info!(?output);
            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityCreateReply {
                    object: Some(output),
                },
            ))
        }
        .instrument(info_span!("kubernetes_deployment_entity_create"))
        .await
    }

    async fn kubernetes_deployment_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityListReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(&self.db, &request, "kubernetes_deployment_entity_list")
                .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let list_reply =
                crate::protobuf::KubernetesDeploymentEntity::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("kubernetes_deployment_entity_list"))
        .await
    }

    async fn kubernetes_deployment_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityGetReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(&self.db, &request, "kubernetes_deployment_entity_get")
                .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let output =
                crate::protobuf::KubernetesDeploymentEntity::get(&self.db, &request_id).await?;
            info!(?output);
            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityGetReply {
                    object: Some(output),
                },
            ))
        }
        .instrument(info_span!("kubernetes_deployment_entity_get"))
        .await
    }

    async fn kubernetes_deployment_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentEntitySyncRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntitySyncReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(&self.db, &request, "kubernetes_deployment_entity_sync")
                .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let output =
                crate::model::KubernetesDeploymentEntity::kubernetes_deployment_entity_sync(
                    &self.db, inner,
                )
                .await?;
            info!(?output);
            Ok(tonic::Response::new(output))
        }
        .instrument(info_span!("kubernetes_deployment_entity_sync"))
        .await
    }

    async fn kubernetes_deployment_entity_kubernetes_object_edit(
        &self,
        request: tonic::Request<
            crate::protobuf::KubernetesDeploymentEntityKubernetesObjectEditRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityKubernetesObjectEditReply>,
        tonic::Status,
    > {
        async {
      info!(?request);
        si_account::authorize::authnz(&self.db, &request, "kubernetes_deployment_entity_kubernetes_object_edit").await?;         
      // The request always gets consumed!
      let inner = request.into_inner();
        let output = crate::model::KubernetesDeploymentEntity::kubernetes_deployment_entity_kubernetes_object_edit(&self.db, inner).await?;
        info!(?output);
        Ok(tonic::Response::new(output))
    }
    .instrument(info_span!("kubernetes_deployment_entity_kubernetes_object_edit"))
    .await
    }

    async fn kubernetes_deployment_entity_kubernetes_object_yaml_edit(
        &self,
        request: tonic::Request<
            crate::protobuf::KubernetesDeploymentEntityKubernetesObjectYamlEditRequest,
        >,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityKubernetesObjectYamlEditReply>,
        tonic::Status,
    > {
        async {
      info!(?request);
        si_account::authorize::authnz(&self.db, &request, "kubernetes_deployment_entity_kubernetes_object_yaml_edit").await?;         
      // The request always gets consumed!
      let inner = request.into_inner();
        let output = crate::model::KubernetesDeploymentEntity::kubernetes_deployment_entity_kubernetes_object_yaml_edit(&self.db, inner).await?;
        info!(?output);
        Ok(tonic::Response::new(output))
    }
    .instrument(info_span!("kubernetes_deployment_entity_kubernetes_object_yaml_edit"))
    .await
    }

    async fn kubernetes_deployment_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::KubernetesDeploymentEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::KubernetesDeploymentEntityEventListReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            si_account::authorize::authnz(
                &self.db,
                &request,
                "kubernetes_deployment_entity_event_list",
            )
            .await?;
            // The request always gets consumed!
            let inner = request.into_inner();
            let list_reply =
                crate::protobuf::KubernetesDeploymentEntityEvent::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::KubernetesDeploymentEntityEventListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("kubernetes_deployment_entity_event_list"))
        .await
    }
}
