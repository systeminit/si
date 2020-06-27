// Auto-generated rust code!
// No-Touchy!

use opentelemetry::api::HttpTextFormat;
use tracing_futures::Instrument as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub use crate::protobuf::core_server::CoreServer as Server;

#[derive(Debug)]
pub struct Service {
    db: si_data::Db,
    agent: si_cea::AgentClient,
}

impl Service {
    pub fn new(db: si_data::Db, agent: si_cea::AgentClient) -> Service {
        Service { db, agent }
    }

    pub async fn migrate(&self) -> si_data::Result<()> {
        crate::protobuf::ServiceComponent::migrate(&self.db).await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl crate::protobuf::core_server::Core for Service {
    async fn service_component_create(
        &self,
        request: tonic::Request<crate::protobuf::ServiceComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceComponentCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_component_create",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_component_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::ServiceComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_component_get(
        &self,
        request: tonic::Request<crate::protobuf::ServiceComponentGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceComponentGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_component_get",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ServiceComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_component_list(
        &self,
        request: tonic::Request<crate::protobuf::ServiceComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceComponentListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_component_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_component_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::ServiceComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::ServiceComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceComponentPickReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_component_pick",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_component_pick").await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::ServiceComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_create",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_entity_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let workspace_id = inner.workspace_id;
            let change_set_id = inner.change_set_id;
            let properties = inner.properties;
            let constraints = inner.constraints;

            let workspace_id = workspace_id.ok_or_else(|| {
                si_data::DataError::ValidationError(
                    "missing required workspace_id value".to_string(),
                )
            })?;

            let workspace = si_account::Workspace::get(&self.db, &workspace_id).await?;

            let (implicit_constraints, component) =
                crate::protobuf::ServiceComponent::pick(&self.db, constraints.clone()).await?;

            let si_properties = si_cea::EntitySiProperties::new(
                &workspace,
                component
                    .id
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?,
                component.si_properties.as_ref().ok_or_else(|| {
                    si_data::DataError::RequiredField("si_properties".to_string())
                })?,
            )?;

            let entity = crate::protobuf::ServiceEntity::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                Some(implicit_constraints),
                properties,
                Some(si_properties),
                change_set_id,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityDeleteRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityDeleteReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_delete",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_entity_delete").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_deploy(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityDeployRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityDeployReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_deploy",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_deploy").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServiceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "deploy",
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityDeployReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServiceEntityGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_get",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_image_edit(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityImageEditRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityImageEditReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_image_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_image_edit")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_image(property)?;
            entity.save(&self.db).await?;

            let entity_event = crate::protobuf::ServiceEntityEvent::create_with_previous_entity(
                &self.db,
                auth.user_id(),
                "edit_image",
                &entity,
                previous_entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityImageEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServiceEntityListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ServiceEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_port_edit(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityPortEditRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityPortEditReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_port_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_port_edit")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_port(property)?;
            entity.save(&self.db).await?;

            let entity_event = crate::protobuf::ServiceEntityEvent::create_with_previous_entity(
                &self.db,
                auth.user_id(),
                "edit_port",
                &entity,
                previous_entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityPortEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_replicas_edit(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityReplicasEditRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityReplicasEditReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_replicas_edit",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::{Entity, EntityEvent};

            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_replicas_edit")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let property = inner
                .property
                .ok_or_else(|| si_data::DataError::RequiredField("property".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let previous_entity = entity.clone();

            entity.set_entity_state_transition();
            entity.edit_replicas(property)?;
            entity.save(&self.db).await?;

            let entity_event = crate::protobuf::ServiceEntityEvent::create_with_previous_entity(
                &self.db,
                auth.user_id(),
                "edit_replicas",
                &entity,
                previous_entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityReplicasEditReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntitySyncRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServiceEntitySyncReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_sync",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            use si_cea::EntityEvent;

            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_sync").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServiceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityUpdateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityUpdateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_update",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            si_account::authorize::authnz(&self.db, &request, "service_entity_update").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityEventListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_event_list",
            metadata.content_type = tracing::field::Empty,
            authenticated = tracing::field::Empty,
            userId = tracing::field::Empty,
            billingAccountId = tracing::field::Empty,
            http.user_agent = tracing::field::Empty,
        );
        span.set_parent(&span_context);

        {
            let metadata = request.metadata();
            if let Some(raw_value) = metadata.get("authenticated") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("authenticated", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("userid") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("userId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("billingAccountId") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("billingAccountId", &tracing::field::display(value));
            }
            if let Some(raw_value) = metadata.get("user-agent") {
                let value = raw_value.to_str().unwrap_or("unserializable");
                span.record("http.user_agent", &tracing::field::display(value));
            }
        }

        async {
            #[allow(unused_variables)]
            let auth =
                si_account::authorize::authnz(&self.db, &request, "service_entity_event_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ServiceEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityEventListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }
}
