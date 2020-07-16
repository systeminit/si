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
        crate::protobuf::ApplicationComponent::migrate(&self.db).await?;
        crate::protobuf::ServiceComponent::migrate(&self.db).await?;
        crate::protobuf::SystemComponent::migrate(&self.db).await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl crate::protobuf::core_server::Core for Service {
    async fn application_component_create(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationComponentCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_component_create",
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
            si_account::authorize::authnz(&self.db, &request, "application_component_create")
                .await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::ApplicationComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_component_get(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationComponentGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationComponentGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_component_get",
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
            si_account::authorize::authnz(&self.db, &request, "application_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ApplicationComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_component_list(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationComponentListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_component_list",
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
                si_account::authorize::authnz(&self.db, &request, "application_component_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::ApplicationComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationComponentPickReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_component_pick",
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
            si_account::authorize::authnz(&self.db, &request, "application_component_pick").await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::ApplicationComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_create",
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
            si_account::authorize::authnz(&self.db, &request, "application_entity_create").await?;

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
                crate::protobuf::ApplicationComponent::pick(&self.db, constraints.clone()).await?;

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

            let entity = crate::protobuf::ApplicationEntity::create(
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
                crate::protobuf::ApplicationEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityDeleteRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityDeleteReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_delete",
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
            si_account::authorize::authnz(&self.db, &request, "application_entity_delete").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ApplicationEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_get",
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
            si_account::authorize::authnz(&self.db, &request, "application_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ApplicationEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_list",
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
            let auth = si_account::authorize::authnz(&self.db, &request, "application_entity_list")
                .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ApplicationEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntitySyncRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntitySyncReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_sync",
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

            let auth = si_account::authorize::authnz(&self.db, &request, "application_entity_sync")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::ApplicationEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ApplicationEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityUpdateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityUpdateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_update",
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
            si_account::authorize::authnz(&self.db, &request, "application_entity_update").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ApplicationEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityEventListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_event_list",
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
                si_account::authorize::authnz(&self.db, &request, "application_entity_event_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ApplicationEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityEventListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

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
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServiceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "deploy",
                &change_set_id,
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
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServiceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
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

    async fn system_component_create(
        &self,
        request: tonic::Request<crate::protobuf::SystemComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::SystemComponentCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_component_create",
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
            si_account::authorize::authnz(&self.db, &request, "system_component_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::SystemComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_component_get(
        &self,
        request: tonic::Request<crate::protobuf::SystemComponentGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemComponentGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_component_get",
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
            si_account::authorize::authnz(&self.db, &request, "system_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::SystemComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_component_list(
        &self,
        request: tonic::Request<crate::protobuf::SystemComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::SystemComponentListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_component_list",
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
                si_account::authorize::authnz(&self.db, &request, "system_component_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::SystemComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::SystemComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::SystemComponentPickReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_component_pick",
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
            si_account::authorize::authnz(&self.db, &request, "system_component_pick").await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::SystemComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntityCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemEntityCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_create",
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
            si_account::authorize::authnz(&self.db, &request, "system_entity_create").await?;

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
                crate::protobuf::SystemComponent::pick(&self.db, constraints.clone()).await?;

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

            let entity = crate::protobuf::SystemEntity::create(
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
                crate::protobuf::SystemEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntityDeleteRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemEntityDeleteReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_delete",
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
            si_account::authorize::authnz(&self.db, &request, "system_entity_delete").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::SystemEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntityGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemEntityGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_get",
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
            si_account::authorize::authnz(&self.db, &request, "system_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::SystemEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntityListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemEntityListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_list",
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
                si_account::authorize::authnz(&self.db, &request, "system_entity_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::SystemEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntitySyncRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemEntitySyncReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_sync",
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
                si_account::authorize::authnz(&self.db, &request, "system_entity_sync").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::SystemEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::SystemEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntityUpdateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemEntityUpdateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_update",
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
            si_account::authorize::authnz(&self.db, &request, "system_entity_update").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::SystemEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn system_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::SystemEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::SystemEntityEventListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_entity_event_list",
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
                si_account::authorize::authnz(&self.db, &request, "system_entity_event_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::SystemEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::SystemEntityEventListReply {
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
