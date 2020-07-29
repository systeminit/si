// Auto-generated rust code!
// No-Touchy!

use opentelemetry::api::HttpTextFormat;
use tracing_futures::Instrument as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub use crate::protobuf::account_server::AccountServer as Server;

#[derive(Debug)]
pub struct Service {
    db: si_data::Db,
}

impl Service {
    pub fn new(db: si_data::Db) -> Service {
        Service { db }
    }

    pub async fn migrate(&self) -> si_data::Result<()> {
        crate::protobuf::Integration::migrate(&self.db).await?;
        crate::protobuf::IntegrationService::migrate(&self.db).await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl crate::protobuf::account_server::Account for Service {
    async fn billing_account_create(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::BillingAccountCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.billing_account_create",
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
            crate::authorize::authnz(&self.db, &request, "billing_account_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;

            let output =
                crate::protobuf::BillingAccount::create(&self.db, name, display_name).await?;

            Ok(tonic::Response::new(
                crate::protobuf::BillingAccountCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn billing_account_get(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::BillingAccountGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.billing_account_get",
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
            crate::authorize::authnz(&self.db, &request, "billing_account_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::BillingAccount::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::BillingAccountGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn billing_account_list(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::BillingAccountListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.billing_account_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "billing_account_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::BillingAccount::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::BillingAccountListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn billing_account_signup(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountSignupRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::BillingAccountSignupReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.billing_account_signup",
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
            // Authentication is skipped on `billing_account_signup`

            let inner = request.into_inner();

            let output = crate::model::BillingAccount::signup(&self.db, inner).await?;

            Ok(tonic::Response::new(output))
        }
        .instrument(span)
        .await
    }

    async fn change_set_create(
        &self,
        request: tonic::Request<crate::protobuf::ChangeSetCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ChangeSetCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.change_set_create",
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
            crate::authorize::authnz(&self.db, &request, "change_set_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let note = inner.note;
            let workspace_id = inner.workspace_id;
            let created_by_user_id = inner.created_by_user_id;

            let real_workspace_id = workspace_id.clone().ok_or_else(|| {
                si_data::DataError::ValidationError(
                    "missing required workspace_id value".to_string(),
                )
            })?;

            let workspace = crate::protobuf::Workspace::get(&self.db, &real_workspace_id).await?;
            let organization_id = Some(
                workspace
                    .si_properties
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
                    .organization_id
                    .as_ref()
                    .ok_or_else(|| {
                        si_data::DataError::RequiredField("organization_id".to_string())
                    })?
                    .clone(),
            );
            let billing_account_id = Some(
                workspace
                    .si_properties
                    .as_ref()
                    .ok_or_else(|| si_data::DataError::RequiredField("si_properties".to_string()))?
                    .billing_account_id
                    .as_ref()
                    .ok_or_else(|| {
                        si_data::DataError::RequiredField("billing_account_id".to_string())
                    })?
                    .clone(),
            );

            let si_properties = crate::protobuf::ChangeSetSiProperties {
                workspace_id: Some(real_workspace_id.clone()),
                organization_id,
                billing_account_id,
            };

            let output = crate::protobuf::ChangeSet::create(
                &self.db,
                name,
                display_name,
                note,
                created_by_user_id,
                Some(si_properties),
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::ChangeSetCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn change_set_execute(
        &self,
        request: tonic::Request<crate::protobuf::ChangeSetExecuteRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ChangeSetExecuteReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.change_set_execute",
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
            crate::authorize::authnz(&self.db, &request, "change_set_execute").await?;

            let inner = request.into_inner();

            let output = crate::model::ChangeSet::execute(&self.db, inner).await?;

            Ok(tonic::Response::new(output))
        }
        .instrument(span)
        .await
    }

    async fn change_set_get(
        &self,
        request: tonic::Request<crate::protobuf::ChangeSetGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ChangeSetGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.change_set_get",
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
            crate::authorize::authnz(&self.db, &request, "change_set_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::ChangeSet::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::ChangeSetGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn change_set_list(
        &self,
        request: tonic::Request<crate::protobuf::ChangeSetListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ChangeSetListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.change_set_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "change_set_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::ChangeSet::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::ChangeSetListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn event_log_create(
        &self,
        request: tonic::Request<crate::protobuf::EventLogCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::EventLogCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.event_log_create",
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
            crate::authorize::authnz(&self.db, &request, "event_log_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let level = crate::protobuf::EventLogLevel::from_i32(inner.level);
            let si_properties = inner.si_properties;
            let message = inner.message;
            let payload = inner.payload;
            let related_ids = inner.related_ids;
            let timestamp = inner.timestamp;
            let created_by_user_id = inner.created_by_user_id;

            let output = crate::protobuf::EventLog::create(
                &self.db,
                name,
                display_name,
                level,
                si_properties,
                message,
                payload,
                related_ids,
                timestamp,
                created_by_user_id,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::EventLogCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn event_log_get(
        &self,
        request: tonic::Request<crate::protobuf::EventLogGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::EventLogGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.event_log_get",
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
            crate::authorize::authnz(&self.db, &request, "event_log_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::EventLog::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::EventLogGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn event_log_list(
        &self,
        request: tonic::Request<crate::protobuf::EventLogListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::EventLogListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.event_log_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "event_log_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::EventLog::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::EventLogListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn group_create(
        &self,
        request: tonic::Request<crate::protobuf::GroupCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::GroupCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.group_create",
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
            crate::authorize::authnz(&self.db, &request, "group_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let user_ids = inner.user_ids;
            let si_properties = inner.si_properties;
            let capabilities = inner.capabilities;

            let output = crate::protobuf::Group::create(
                &self.db,
                name,
                display_name,
                user_ids,
                si_properties,
                capabilities,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::GroupCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn group_get(
        &self,
        request: tonic::Request<crate::protobuf::GroupGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::GroupGetReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.group_get",
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
            crate::authorize::authnz(&self.db, &request, "group_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Group::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::GroupGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn group_list(
        &self,
        request: tonic::Request<crate::protobuf::GroupListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::GroupListReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.group_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "group_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Group::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::GroupListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn integration_create(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::IntegrationCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_create",
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
            crate::authorize::authnz(&self.db, &request, "integration_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let options = inner.options;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::Integration::create(
                &self.db,
                name,
                display_name,
                options,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_get(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::IntegrationGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_get",
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
            crate::authorize::authnz(&self.db, &request, "integration_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Integration::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::IntegrationGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn integration_list(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::IntegrationListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "integration_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::Integration::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_instance_create(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationInstanceCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationInstanceCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_instance_create",
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
            crate::authorize::authnz(&self.db, &request, "integration_instance_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let option_values = inner.option_values;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::IntegrationInstance::create(
                &self.db,
                name,
                display_name,
                option_values,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationInstanceCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_instance_get(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationInstanceGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationInstanceGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_instance_get",
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
            crate::authorize::authnz(&self.db, &request, "integration_instance_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::IntegrationInstance::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationInstanceGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_instance_list(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationInstanceListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationInstanceListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_instance_list",
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
                crate::authorize::authnz(&self.db, &request, "integration_instance_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::IntegrationInstance::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationInstanceListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_service_create(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationServiceCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationServiceCreateReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_service_create",
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
            crate::authorize::authnz(&self.db, &request, "integration_service_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::IntegrationService::create(
                &self.db,
                name,
                display_name,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationServiceCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_service_get(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationServiceGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationServiceGetReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_service_get",
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
            crate::authorize::authnz(&self.db, &request, "integration_service_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::IntegrationService::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationServiceGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn integration_service_list(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationServiceListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationServiceListReply>,
        tonic::Status,
    > {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.integration_service_list",
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
                crate::authorize::authnz(&self.db, &request, "integration_service_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let output = crate::protobuf::IntegrationService::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::IntegrationServiceListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn item_get(
        &self,
        request: tonic::Request<crate::protobuf::ItemGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ItemGetReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.item_get",
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
            crate::authorize::authnz(&self.db, &request, "item_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Item::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::ItemGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn item_list(
        &self,
        request: tonic::Request<crate::protobuf::ItemListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::ItemListReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.item_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "item_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Item::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::ItemListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn organization_create(
        &self,
        request: tonic::Request<crate::protobuf::OrganizationCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::OrganizationCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.organization_create",
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
            crate::authorize::authnz(&self.db, &request, "organization_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;

            let output =
                crate::protobuf::Organization::create(&self.db, name, display_name, si_properties)
                    .await?;

            Ok(tonic::Response::new(
                crate::protobuf::OrganizationCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn organization_get(
        &self,
        request: tonic::Request<crate::protobuf::OrganizationGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::OrganizationGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.organization_get",
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
            crate::authorize::authnz(&self.db, &request, "organization_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Organization::get(&self.db, &id).await?;

            Ok(tonic::Response::new(
                crate::protobuf::OrganizationGetReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn organization_list(
        &self,
        request: tonic::Request<crate::protobuf::OrganizationListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::OrganizationListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.organization_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "organization_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Organization::list(&self.db, inner).await?;

            Ok(tonic::Response::new(
                crate::protobuf::OrganizationListReply {
                    items: output.items,
                    total_count: Some(output.total_count),
                    next_page_token: Some(output.page_token),
                },
            ))
        }
        .instrument(span)
        .await
    }

    async fn user_create(
        &self,
        request: tonic::Request<crate::protobuf::UserCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserCreateReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.user_create",
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
            crate::authorize::authnz(&self.db, &request, "user_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let email = inner.email;
            let password = inner.password;
            let si_properties = inner.si_properties;

            let output = crate::protobuf::User::create(
                &self.db,
                name,
                display_name,
                email,
                password,
                si_properties,
            )
            .await?;

            Ok(tonic::Response::new(crate::protobuf::UserCreateReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn user_get(
        &self,
        request: tonic::Request<crate::protobuf::UserGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserGetReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.user_get",
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
            crate::authorize::authnz(&self.db, &request, "user_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::User::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::UserGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn user_list(
        &self,
        request: tonic::Request<crate::protobuf::UserListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserListReply>, tonic::Status> {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.user_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "user_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::User::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::UserListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }

    async fn user_login_internal(
        &self,
        request: tonic::Request<crate::protobuf::UserLoginInternalRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserLoginInternalReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.user_login_internal",
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
            // Authentication is skipped on `user_login_internal`

            let inner = request.into_inner();

            let output = crate::model::User::login_internal(&self.db, inner).await?;

            Ok(tonic::Response::new(output))
        }
        .instrument(span)
        .await
    }

    async fn workspace_create(
        &self,
        request: tonic::Request<crate::protobuf::WorkspaceCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::WorkspaceCreateReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.workspace_create",
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
            crate::authorize::authnz(&self.db, &request, "workspace_create").await?;

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;

            let output =
                crate::protobuf::Workspace::create(&self.db, name, display_name, si_properties)
                    .await?;

            Ok(tonic::Response::new(
                crate::protobuf::WorkspaceCreateReply { item: Some(output) },
            ))
        }
        .instrument(span)
        .await
    }

    async fn workspace_get(
        &self,
        request: tonic::Request<crate::protobuf::WorkspaceGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::WorkspaceGetReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.workspace_get",
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
            crate::authorize::authnz(&self.db, &request, "workspace_get").await?;

            let inner = request.into_inner();
            let id = inner
                .id
                .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?;

            let output = crate::protobuf::Workspace::get(&self.db, &id).await?;

            Ok(tonic::Response::new(crate::protobuf::WorkspaceGetReply {
                item: Some(output),
            }))
        }
        .instrument(span)
        .await
    }

    async fn workspace_list(
        &self,
        request: tonic::Request<crate::protobuf::WorkspaceListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::WorkspaceListReply>, tonic::Status>
    {
        let span_context =
            opentelemetry::api::TraceContextPropagator::new().extract(request.metadata());
        let span = tracing::span!(
            tracing::Level::INFO,
            "account.workspace_list",
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
            let auth = crate::authorize::authnz(&self.db, &request, "workspace_list").await?;

            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let output = crate::protobuf::Workspace::list(&self.db, inner).await?;

            Ok(tonic::Response::new(crate::protobuf::WorkspaceListReply {
                items: output.items,
                total_count: Some(output.total_count),
                next_page_token: Some(output.page_token),
            }))
        }
        .instrument(span)
        .await
    }
}
