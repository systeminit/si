// Auto-generated rust code!
// No-Touchy!

use opentelemetry::api::HttpTextFormat;
use tracing_futures::Instrument as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub use crate::protobuf::core_server::CoreServer as Server;

#[derive(Debug)]
pub struct Service {
    db: si_data::Db,
}

impl Service {
    pub fn new(db: si_data::Db) -> Service {
        Service { db }
    }

    pub async fn migrate(&self) -> si_data::Result<()> {
        crate::protobuf::ApplicationComponent::migrate(&self.db).await?;
        crate::protobuf::ServiceComponent::migrate(&self.db).await?;
        crate::protobuf::ServerComponent::migrate(&self.db).await?;
        crate::protobuf::UbuntuComponent::migrate(&self.db).await?;
        crate::protobuf::AmiComponent::migrate(&self.db).await?;
        crate::protobuf::Ec2InstanceComponent::migrate(&self.db).await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl crate::protobuf::core_server::Core for Service {
    async fn ami_component_create(
        &self,
        request: tonic::Request<crate::protobuf::AmiComponentCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiComponentCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_component_create",
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
            si_account::authorize::authnz(&self.db, &request, "ami_component_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::AmiComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_component_get(
        &self,
        request: tonic::Request<crate::protobuf::AmiComponentGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiComponentGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_component_get",
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
            si_account::authorize::authnz(&self.db, &request, "ami_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::AmiComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_component_list(
        &self,
        request: tonic::Request<crate::protobuf::AmiComponentListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiComponentListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_component_list",
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
                si_account::authorize::authnz(&self.db, &request, "ami_component_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::AmiComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::AmiComponentPickRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiComponentPickReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_component_pick",
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
            si_account::authorize::authnz(&self.db, &request, "ami_component_pick").await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::AmiComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_create",
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
            si_account::authorize::authnz(&self.db, &request, "ami_entity_create").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

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
                crate::protobuf::AmiComponent::pick(&self.db, constraints.clone()).await?;

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

            let entity = crate::protobuf::AmiEntity::create(
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

            si_account::EventLog::entity_create(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityDeleteRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityDeleteReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_delete",
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
            si_account::authorize::authnz(&self.db, &request, "ami_entity_delete").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::AmiEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            si_account::EventLog::entity_delete(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_get",
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
            si_account::authorize::authnz(&self.db, &request, "ami_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::AmiEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::AmiEntityGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_list",
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
            let auth = si_account::authorize::authnz(&self.db, &request, "ami_entity_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::AmiEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::AmiEntityListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntitySyncRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntitySyncReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_sync",
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

            let auth = si_account::authorize::authnz(&self.db, &request, "ami_entity_sync").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::AmiEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::AmiEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::AmiEntitySyncReply {
                item: Some(entity_event),
            }))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityUpdateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityUpdateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_update",
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
            si_account::authorize::authnz(&self.db, &request, "ami_entity_update").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::AmiEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            si_account::EventLog::entity_update(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_event_get(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityEventGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityEventGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_event_get",
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
            si_account::authorize::authnz(&self.db, &request, "ami_entity_event_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::AmiEntityEvent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiEntityEventGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ami_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::AmiEntityEventListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::AmiEntityEventListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ami_entity_event_list",
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
                si_account::authorize::authnz(&self.db, &request, "ami_entity_event_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::AmiEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::AmiEntityEventListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

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

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

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

            si_account::EventLog::entity_create(&self.db, &user_id, &entity).await?;

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

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ApplicationEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            si_account::EventLog::entity_delete(&self.db, &user_id, &entity).await?;

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

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ApplicationEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            si_account::EventLog::entity_update(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn application_entity_event_get(
        &self,
        request: tonic::Request<crate::protobuf::ApplicationEntityEventGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ApplicationEntityEventGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.application_entity_event_get",
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
            si_account::authorize::authnz(&self.db, &request, "application_entity_event_get")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ApplicationEntityEvent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ApplicationEntityEventGetReply { item: Some(output) },
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

    async fn ec2_instance_component_create(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceComponentCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_component_create",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_component_create")
                .await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::Ec2InstanceComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_component_get(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceComponentGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceComponentGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_component_get",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Ec2InstanceComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_component_list(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceComponentListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_component_list",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_component_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::Ec2InstanceComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceComponentPickReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_component_pick",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_component_pick")
                .await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::Ec2InstanceComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_create",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_create").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

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
                crate::protobuf::Ec2InstanceComponent::pick(&self.db, constraints.clone()).await?;

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

            let entity = crate::protobuf::Ec2InstanceEntity::create(
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

            si_account::EventLog::entity_create(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityDeleteRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityDeleteReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_delete",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_delete").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            si_account::EventLog::entity_delete(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_get",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_launch(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityLaunchRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityLaunchReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_launch",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_launch")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::Ec2InstanceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "launch",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityLaunchReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_list",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Ec2InstanceEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_reboot(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityRebootRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityRebootReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_reboot",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_reboot")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::Ec2InstanceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "reboot",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityRebootReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_start(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityStartRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityStartReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_start",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_start")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::Ec2InstanceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "start",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityStartReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_stop(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityStopRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityStopReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_stop",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_stop")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::Ec2InstanceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "stop",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityStopReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntitySyncRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntitySyncReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_sync",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_sync")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::Ec2InstanceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_terminate(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityTerminateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityTerminateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_terminate",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_terminate")
                    .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::Ec2InstanceEntityEvent::create(
                &self.db,
                auth.user_id(),
                "terminate",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityTerminateReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityUpdateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityUpdateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_update",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_update").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::Ec2InstanceEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            si_account::EventLog::entity_update(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_event_get(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityEventGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityEventGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_event_get",
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
            si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_event_get")
                .await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Ec2InstanceEntityEvent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityEventGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ec2_instance_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::Ec2InstanceEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::Ec2InstanceEntityEventListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ec2_instance_entity_event_list",
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
                si_account::authorize::authnz(&self.db, &request, "ec2_instance_entity_event_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Ec2InstanceEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::Ec2InstanceEntityEventListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn edge_create(
        &self,
        request: tonic::Request<crate::protobuf::EdgeCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::EdgeCreateReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.edge_create",
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
            si_account::authorize::authnz(&self.db, &request, "edge_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;
            let head_vertex = inner.head_vertex;
            let tail_vertex = inner.tail_vertex;
            let bidirectional = inner.bidirectional;
            let edge_kind = crate::protobuf::EdgeEdgeKind::from_i32(inner.edge_kind);

            let output = crate::protobuf::Edge::create(
                &self.db,
                name,
                display_name,
                si_properties,
                head_vertex,
                tail_vertex,
                bidirectional,
                edge_kind,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::EdgeCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn edge_get(
        &self,
        request: tonic::Request<crate::protobuf::EdgeGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::EdgeGetReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.edge_get",
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
            si_account::authorize::authnz(&self.db, &request, "edge_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Edge::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::EdgeGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn edge_list(
        &self,
        request: tonic::Request<crate::protobuf::EdgeListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::EdgeListReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.edge_list",
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
            let auth = si_account::authorize::authnz(&self.db, &request, "edge_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Edge::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::EdgeListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn node_create(
        &self,
        request: tonic::Request<crate::protobuf::NodeCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::NodeCreateReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.node_create",
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
            si_account::authorize::authnz(&self.db, &request, "node_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let entity_id = inner.entity_id;
            let sockets = inner.sockets;
            let position = inner.position;
            let node_kind = crate::protobuf::NodeNodeKind::from_i32(inner.node_kind);
            let si_properties = inner.si_properties;

            let output = crate::protobuf::Node::create(
                &self.db,
                name,
                display_name,
                entity_id,
                sockets,
                position,
                node_kind,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::NodeCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn node_get(
        &self,
        request: tonic::Request<crate::protobuf::NodeGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::NodeGetReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.node_get",
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
            si_account::authorize::authnz(&self.db, &request, "node_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Node::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::NodeGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn node_list(
        &self,
        request: tonic::Request<crate::protobuf::NodeListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::NodeListReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.node_list",
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
            let auth = si_account::authorize::authnz(&self.db, &request, "node_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Node::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::NodeListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn node_set_position(
        &self,
        request: tonic::Request<crate::protobuf::NodeSetPositionRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::NodeSetPositionReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.node_set_position",
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
            si_account::authorize::authnz(&self.db, &request, "node_set_position").await?;

            let inner = request.into_inner();

            let output = crate::model::Node::set_position(&self.db, inner).await?;

            Ok(tonic::Response::new(output))
        }
        .instrument(span)
        .await
    }

    async fn resource_create(
        &self,
        request: tonic::Request<crate::protobuf::ResourceCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ResourceCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.resource_create",
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
            si_account::authorize::authnz(&self.db, &request, "resource_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;
            let entity_id = inner.entity_id;
            let node_id = inner.node_id;
            let kind = inner.kind;
            let status = inner.status;
            let health = inner.health;
            let data = inner.data;

            let output = crate::protobuf::Resource::create(
                &self.db,
                name,
                display_name,
                si_properties,
                entity_id,
                node_id,
                kind,
                status,
                health,
                data,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::ResourceCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn resource_get(
        &self,
        request: tonic::Request<crate::protobuf::ResourceGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ResourceGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.resource_get",
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
            si_account::authorize::authnz(&self.db, &request, "resource_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Resource::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::ResourceGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn resource_list(
        &self,
        request: tonic::Request<crate::protobuf::ResourceListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ResourceListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.resource_list",
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
            let auth = si_account::authorize::authnz(&self.db, &request, "resource_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Resource::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::ResourceListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn resource_update(
        &self,
        request: tonic::Request<crate::protobuf::ResourceUpdateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ResourceUpdateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.resource_update",
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
            si_account::authorize::authnz(&self.db, &request, "resource_update").await?;

            let inner = request.into_inner();

            let output = crate::model::Resource::update(&self.db, inner).await?;

            Ok(tonic::Response::new(output))
        }
        .instrument(span)
        .await
    }

    async fn server_component_create(
        &self,
        request: tonic::Request<crate::protobuf::ServerComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServerComponentCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_component_create",
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
            si_account::authorize::authnz(&self.db, &request, "server_component_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::ServerComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_component_get(
        &self,
        request: tonic::Request<crate::protobuf::ServerComponentGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerComponentGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_component_get",
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
            si_account::authorize::authnz(&self.db, &request, "server_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ServerComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_component_list(
        &self,
        request: tonic::Request<crate::protobuf::ServerComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServerComponentListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_component_list",
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
                si_account::authorize::authnz(&self.db, &request, "server_component_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::ServerComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::ServerComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServerComponentPickReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_component_pick",
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
            si_account::authorize::authnz(&self.db, &request, "server_component_pick").await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::ServerComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_create",
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
            si_account::authorize::authnz(&self.db, &request, "server_entity_create").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

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
                crate::protobuf::ServerComponent::pick(&self.db, constraints.clone()).await?;

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

            let entity = crate::protobuf::ServerEntity::create(
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

            si_account::EventLog::entity_create(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityDeleteRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityDeleteReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_delete",
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
            si_account::authorize::authnz(&self.db, &request, "server_entity_delete").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ServerEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            si_account::EventLog::entity_delete(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_get",
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
            si_account::authorize::authnz(&self.db, &request, "server_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ServerEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_list",
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
                si_account::authorize::authnz(&self.db, &request, "server_entity_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ServerEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_start(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityStartRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityStartReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_start",
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
                si_account::authorize::authnz(&self.db, &request, "server_entity_start").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::ServerEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServerEntityEvent::create(
                &self.db,
                auth.user_id(),
                "start",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityStartReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_stop(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityStopRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityStopReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_stop",
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
                si_account::authorize::authnz(&self.db, &request, "server_entity_stop").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::ServerEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServerEntityEvent::create(
                &self.db,
                auth.user_id(),
                "stop",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityStopReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntitySyncRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntitySyncReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_sync",
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
                si_account::authorize::authnz(&self.db, &request, "server_entity_sync").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::ServerEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::ServerEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityUpdateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ServerEntityUpdateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_update",
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
            si_account::authorize::authnz(&self.db, &request, "server_entity_update").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ServerEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            si_account::EventLog::entity_update(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_event_get(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityEventGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServerEntityEventGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_event_get",
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
            si_account::authorize::authnz(&self.db, &request, "server_entity_event_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ServerEntityEvent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityEventGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn server_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::ServerEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServerEntityEventListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.server_entity_event_list",
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
                si_account::authorize::authnz(&self.db, &request, "server_entity_event_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ServerEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServerEntityEventListReply {
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

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

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

            si_account::EventLog::entity_create(&self.db, &user_id, &entity).await?;

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

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            si_account::EventLog::entity_delete(&self.db, &user_id, &entity).await?;

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

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::ServiceEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            si_account::EventLog::entity_update(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn service_entity_event_get(
        &self,
        request: tonic::Request<crate::protobuf::ServiceEntityEventGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::ServiceEntityEventGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.service_entity_event_get",
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
            si_account::authorize::authnz(&self.db, &request, "service_entity_event_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ServiceEntityEvent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::ServiceEntityEventGetReply { item: Some(output) },
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

    async fn system_create(
        &self,
        request: tonic::Request<crate::protobuf::SystemCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_create",
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
            si_account::authorize::authnz(&self.db, &request, "system_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;

            let output =
                crate::protobuf::System::create(&self.db, name, display_name, si_properties)
                    .await?;

            Ok(tonic::Response::new(crate::protobuf::SystemCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn system_get(
        &self,
        request: tonic::Request<crate::protobuf::SystemGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemGetReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_get",
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
            si_account::authorize::authnz(&self.db, &request, "system_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::System::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::SystemGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn system_list(
        &self,
        request: tonic::Request<crate::protobuf::SystemListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::SystemListReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.system_list",
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
            let auth = si_account::authorize::authnz(&self.db, &request, "system_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::System::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::SystemListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_component_create(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuComponentCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::UbuntuComponentCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_component_create",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_component_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let description = inner.description;
            let constraints = inner.constraints;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::UbuntuComponent::create(
                &self.db,
                name,
                display_name,
                description,
                constraints,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuComponentCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_component_get(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuComponentGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuComponentGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_component_get",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_component_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::UbuntuComponent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuComponentGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_component_list(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuComponentListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::UbuntuComponentListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_component_list",
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
                si_account::authorize::authnz(&self.db, &request, "ubuntu_component_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::UbuntuComponent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuComponentListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_component_pick(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuComponentPickRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::UbuntuComponentPickReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_component_pick",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_component_pick").await?;

            let inner = request.into_inner();
            let constraints = inner.constraints;

            let (implicit_constraints, component) =
                crate::protobuf::UbuntuComponent::pick(&self.db, constraints).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuComponentPickReply {
                    implicit_constraints: Some(implicit_constraints),
                    component: Some(component),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_create(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_create",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_create").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

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
                crate::protobuf::UbuntuComponent::pick(&self.db, constraints.clone()).await?;

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

            let entity = crate::protobuf::UbuntuEntity::create(
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

            si_account::EventLog::entity_create(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityCreateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_delete(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityDeleteRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityDeleteReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_delete",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_delete").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::UbuntuEntity::get(&self.db, &id).await?;

            entity.delete(&self.db, inner.change_set_id).await?;

            si_account::EventLog::entity_delete(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityDeleteReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_get(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_get",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::UbuntuEntity::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_halt(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityHaltRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityHaltReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_halt",
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
                si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_halt").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::UbuntuEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::UbuntuEntityEvent::create(
                &self.db,
                auth.user_id(),
                "halt",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityHaltReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_list(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_list",
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
                si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::UbuntuEntity::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_reboot(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityRebootRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityRebootReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_reboot",
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
                si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_reboot").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::UbuntuEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::UbuntuEntityEvent::create(
                &self.db,
                auth.user_id(),
                "reboot",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityRebootReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_sync(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntitySyncRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntitySyncReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_sync",
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
                si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_sync").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;
            let change_set_id = inner
                .change_set_id
                .ok_or_else(|| si_data::DataError::RequiredField("changeSetId".to_string()))?;

            let entity = crate::protobuf::UbuntuEntity::get(&self.db, &id).await?;
            let entity_event = crate::protobuf::UbuntuEntityEvent::create(
                &self.db,
                auth.user_id(),
                "sync",
                &change_set_id,
                &entity,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntitySyncReply {
                    item: Some(entity_event),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_update(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityUpdateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UbuntuEntityUpdateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_update",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_update").await?;

            let user_id = request
                .metadata()
                .get("userid")
                .map(|r| r.to_str().unwrap_or("no_user_id_bug_live_here"))
                .unwrap()
                .to_string();

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let mut entity = crate::protobuf::UbuntuEntity::get(&self.db, &id).await?;

            entity
                .update(&self.db, inner.change_set_id, inner.update)
                .await?;

            si_account::EventLog::entity_update(&self.db, &user_id, &entity).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityUpdateReply { item: Some(entity) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_event_get(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityEventGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::UbuntuEntityEventGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_event_get",
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
            si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_event_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::UbuntuEntityEvent::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityEventGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn ubuntu_entity_event_list(
        &self,
        request: tonic::Request<crate::protobuf::UbuntuEntityEventListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::UbuntuEntityEventListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "core.ubuntu_entity_event_list",
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
                si_account::authorize::authnz(&self.db, &request, "ubuntu_entity_event_list")
                    .await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::UbuntuEntityEvent::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::UbuntuEntityEventListReply {
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
