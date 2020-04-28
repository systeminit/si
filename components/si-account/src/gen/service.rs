// Auto-generated rust code!
// No-Touchy!

use tonic;
use tracing::{self, info, info_span};
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
impl crate::protobuf::account_server::Account for Service {
    async fn billing_account_list(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::BillingAccountListReply>, tonic::Status>
    {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth = crate::authorize::authnz(&self.db, &request, "billing_account_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let list_reply = crate::protobuf::BillingAccount::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::BillingAccountListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("billing_account_list"))
        .await
    }

    async fn billing_account_get(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::BillingAccountGetReply>, tonic::Status>
    {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::BillingAccount::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::BillingAccountGetReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("billing_account_get"))
        .await
    }

    async fn billing_account_signup(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountSignupRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::BillingAccountSignupReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let reply =
                crate::model::BillingAccount::billing_account_signup(&self.db, inner).await?;
            info!(?reply);
            Ok(tonic::Response::new(reply))
        }
        .instrument(info_span!("billing_account_signup"))
        .await
    }

    async fn billing_account_create(
        &self,
        request: tonic::Request<crate::protobuf::BillingAccountCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::BillingAccountCreateReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let reply =
                crate::protobuf::BillingAccount::create(&self.db, name, display_name).await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::BillingAccountCreateReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("billing_account_create"))
        .await
    }

    async fn user_list(
        &self,
        request: tonic::Request<crate::protobuf::UserListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserListReply>, tonic::Status> {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth = crate::authorize::authnz(&self.db, &request, "user_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let list_reply = crate::protobuf::User::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(crate::protobuf::UserListReply {
                items: list_reply.items,
                total_count: Some(list_reply.total_count),
                next_page_token: Some(list_reply.page_token),
            }))
        }
        .instrument(info_span!("user_list"))
        .await
    }

    async fn user_get(
        &self,
        request: tonic::Request<crate::protobuf::UserGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserGetReply>, tonic::Status> {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::User::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(crate::protobuf::UserGetReply {
                object: Some(reply),
            }))
        }
        .instrument(info_span!("user_get"))
        .await
    }

    async fn user_login_internal(
        &self,
        request: tonic::Request<crate::protobuf::UserLoginInternalRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserLoginInternalReply>, tonic::Status>
    {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let reply = crate::model::User::user_login_internal(&self.db, inner).await?;
            info!(?reply);
            Ok(tonic::Response::new(reply))
        }
        .instrument(info_span!("user_login_internal"))
        .await
    }

    async fn user_create(
        &self,
        request: tonic::Request<crate::protobuf::UserCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::UserCreateReply>, tonic::Status> {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "user_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let email = inner.email;
            let password = inner.password;
            let si_properties = inner.si_properties;
            let reply = crate::protobuf::User::create(
                &self.db,
                name,
                display_name,
                email,
                password,
                si_properties,
            )
            .await?;
            info!(?reply);
            Ok(tonic::Response::new(crate::protobuf::UserCreateReply {
                object: Some(reply),
            }))
        }
        .instrument(info_span!("user_create"))
        .await
    }

    async fn group_list(
        &self,
        request: tonic::Request<crate::protobuf::GroupListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::GroupListReply>, tonic::Status> {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth = crate::authorize::authnz(&self.db, &request, "group_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let list_reply = crate::protobuf::Group::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(crate::protobuf::GroupListReply {
                items: list_reply.items,
                total_count: Some(list_reply.total_count),
                next_page_token: Some(list_reply.page_token),
            }))
        }
        .instrument(info_span!("group_list"))
        .await
    }

    async fn group_get(
        &self,
        request: tonic::Request<crate::protobuf::GroupGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::GroupGetReply>, tonic::Status> {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::Group::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(crate::protobuf::GroupGetReply {
                object: Some(reply),
            }))
        }
        .instrument(info_span!("group_get"))
        .await
    }

    async fn group_create(
        &self,
        request: tonic::Request<crate::protobuf::GroupCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::GroupCreateReply>, tonic::Status>
    {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "group_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let user_ids = inner.user_ids;
            let si_properties = inner.si_properties;
            let capabilities = inner.capabilities;
            let reply = crate::protobuf::Group::create(
                &self.db,
                name,
                display_name,
                user_ids,
                si_properties,
                capabilities,
            )
            .await?;
            info!(?reply);
            Ok(tonic::Response::new(crate::protobuf::GroupCreateReply {
                object: Some(reply),
            }))
        }
        .instrument(info_span!("group_create"))
        .await
    }

    async fn organization_list(
        &self,
        request: tonic::Request<crate::protobuf::OrganizationListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::OrganizationListReply>, tonic::Status>
    {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth = crate::authorize::authnz(&self.db, &request, "organization_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let list_reply = crate::protobuf::Organization::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::OrganizationListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("organization_list"))
        .await
    }

    async fn organization_get(
        &self,
        request: tonic::Request<crate::protobuf::OrganizationGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::OrganizationGetReply>, tonic::Status>
    {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::Organization::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::OrganizationGetReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("organization_get"))
        .await
    }

    async fn organization_create(
        &self,
        request: tonic::Request<crate::protobuf::OrganizationCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::OrganizationCreateReply>, tonic::Status>
    {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "organization_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;
            let reply =
                crate::protobuf::Organization::create(&self.db, name, display_name, si_properties)
                    .await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::OrganizationCreateReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("organization_create"))
        .await
    }

    async fn workspace_list(
        &self,
        request: tonic::Request<crate::protobuf::WorkspaceListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::WorkspaceListReply>, tonic::Status>
    {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth = crate::authorize::authnz(&self.db, &request, "workspace_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let list_reply = crate::protobuf::Workspace::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(crate::protobuf::WorkspaceListReply {
                items: list_reply.items,
                total_count: Some(list_reply.total_count),
                next_page_token: Some(list_reply.page_token),
            }))
        }
        .instrument(info_span!("workspace_list"))
        .await
    }

    async fn workspace_get(
        &self,
        request: tonic::Request<crate::protobuf::WorkspaceGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::WorkspaceGetReply>, tonic::Status>
    {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::Workspace::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(crate::protobuf::WorkspaceGetReply {
                object: Some(reply),
            }))
        }
        .instrument(info_span!("workspace_get"))
        .await
    }

    async fn workspace_create(
        &self,
        request: tonic::Request<crate::protobuf::WorkspaceCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::WorkspaceCreateReply>, tonic::Status>
    {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "workspace_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let si_properties = inner.si_properties;
            let reply =
                crate::protobuf::Workspace::create(&self.db, name, display_name, si_properties)
                    .await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::WorkspaceCreateReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("workspace_create"))
        .await
    }

    async fn integration_list(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationListRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::IntegrationListReply>, tonic::Status>
    {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth = crate::authorize::authnz(&self.db, &request, "integration_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let list_reply = crate::protobuf::Integration::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("integration_list"))
        .await
    }

    async fn integration_get(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationGetRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::IntegrationGetReply>, tonic::Status>
    {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::Integration::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(crate::protobuf::IntegrationGetReply {
                object: Some(reply),
            }))
        }
        .instrument(info_span!("integration_get"))
        .await
    }

    async fn integration_create(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationCreateRequest>,
    ) -> std::result::Result<tonic::Response<crate::protobuf::IntegrationCreateReply>, tonic::Status>
    {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "integration_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let version = inner.version;
            let options = inner.options;
            let reply = crate::protobuf::Integration::create(
                &self.db,
                name,
                display_name,
                version,
                options,
            )
            .await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationCreateReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("integration_create"))
        .await
    }

    async fn integration_service_list(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationServiceListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationServiceListReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth =
                crate::authorize::authnz(&self.db, &request, "integration_service_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(String::from("global"));
            }

            let list_reply = crate::protobuf::IntegrationService::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationServiceListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("integration_service_list"))
        .await
    }

    async fn integration_service_get(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationServiceGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationServiceGetReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::IntegrationService::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationServiceGetReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("integration_service_get"))
        .await
    }

    async fn integration_service_create(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationServiceCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationServiceCreateReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "integration_service_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let version = inner.version;
            let si_properties = inner.si_properties;
            let reply = crate::protobuf::IntegrationService::create(
                &self.db,
                name,
                display_name,
                version,
                si_properties,
            )
            .await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationServiceCreateReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("integration_service_create"))
        .await
    }

    async fn integration_instance_list(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationInstanceListRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationInstanceListReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            #[allow(unused_variables)]
            let auth =
                crate::authorize::authnz(&self.db, &request, "integration_instance_list").await?;
            let mut inner = request.into_inner();
            if inner.scope_by_tenant_id.is_none() {
                inner.scope_by_tenant_id = Some(auth.billing_account_id().into());
            }

            let list_reply = crate::protobuf::IntegrationInstance::list(&self.db, inner).await?;
            info!(?list_reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationInstanceListReply {
                    items: list_reply.items,
                    total_count: Some(list_reply.total_count),
                    next_page_token: Some(list_reply.page_token),
                },
            ))
        }
        .instrument(info_span!("integration_instance_list"))
        .await
    }

    async fn integration_instance_get(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationInstanceGetRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationInstanceGetReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            // Skipping authentication

            let inner = request.into_inner();
            let request_id = inner
                .id
                .ok_or(si_data::DataError::RequiredField("id".to_string()))?;
            let reply = crate::protobuf::IntegrationInstance::get(&self.db, &request_id).await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationInstanceGetReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("integration_instance_get"))
        .await
    }

    async fn integration_instance_create(
        &self,
        request: tonic::Request<crate::protobuf::IntegrationInstanceCreateRequest>,
    ) -> std::result::Result<
        tonic::Response<crate::protobuf::IntegrationInstanceCreateReply>,
        tonic::Status,
    > {
        async {
            info!(?request);
            crate::authorize::authnz(&self.db, &request, "integration_instance_create").await?;
            let inner = request.into_inner();
            let name = inner.name;
            let display_name = inner.display_name;
            let option_values = inner.option_values;
            let si_properties = inner.si_properties;
            let reply = crate::protobuf::IntegrationInstance::create(
                &self.db,
                name,
                display_name,
                option_values,
                si_properties,
            )
            .await?;
            info!(?reply);
            Ok(tonic::Response::new(
                crate::protobuf::IntegrationInstanceCreateReply {
                    object: Some(reply),
                },
            ))
        }
        .instrument(info_span!("integration_instance_create"))
        .await
    }
}
