use si_data::{Db, ListResult, Storable};
use tonic::{Request, Response};
use tracing::{debug, debug_span};
use tracing_futures::Instrument;

use crate::authorize::{authorize, authorize_by_tenant_id};
use crate::error::{AccountError, TonicResult};
use crate::model::{
    billing_account, group, integration, integration_instance, organization, user, workspace,
};
use crate::protobuf::{self, account_server};

#[derive(Debug)]
pub struct Service {
    db: Db,
}

impl Service {
    pub fn new(db: Db) -> Service {
        Service { db: db }
    }

    pub fn db(&self) -> &Db {
        &self.db
    }
}

#[tonic::async_trait]
impl account_server::Account for Service {
    async fn get_integration_service(
        &self,
        request: Request<protobuf::GetIntegrationServiceRequest>,
    ) -> TonicResult<protobuf::GetIntegrationServiceReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let integration_service: integration::IntegrationService = self
                .db
                .get(req.integration_service_id.to_string())
                .await
                .map_err(|e| {
                    debug!(?e, "integration_service_get_failed");
                    AccountError::IntegrationServiceMissing
                })?;
            debug!(?integration_service, "found");

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            // We auth on billing account, because eventually it will be the
            // place where you decide what you can see globally, I guess? I dunno.
            // Safe enough for now.
            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "read_integration_service",
                &billing_account,
            )
            .await?;

            Ok(Response::new(protobuf::GetIntegrationServiceReply {
                integration_service: Some(integration_service),
            }))
        }
        .instrument(debug_span!("get_integration_service", ?request))
        .await
    }

    async fn get_integration(
        &self,
        request: Request<protobuf::GetIntegrationRequest>,
    ) -> TonicResult<protobuf::GetIntegrationReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let integration: integration::Integration = self
                .db
                .get(req.integration_id.to_string())
                .await
                .map_err(|e| {
                    debug!(?e, "user_get_failed");
                    AccountError::IntegrationMissing
                })?;
            debug!(?integration, "found");

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            // We auth on billing account, because eventually it will be the
            // place where you decide what you can see globally, I guess? I dunno.
            // Safe enough for now.
            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "read_integration",
                &billing_account,
            )
            .await?;

            Ok(Response::new(protobuf::GetIntegrationReply {
                integration: Some(integration),
            }))
        }
        .instrument(debug_span!("get_integration", ?request))
        .await
    }

    async fn get_user(
        &self,
        request: Request<protobuf::GetUserRequest>,
    ) -> TonicResult<protobuf::GetUserReply> {
        async {
            debug!("get_user");
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let user: user::User = self.db.get(req.user_id.to_string()).await.map_err(|e| {
                debug!(?e, "user_get_failed");
                AccountError::UserMissing
            })?;
            debug!(?user, "found");

            authorize(&self.db, user_id, billing_account_id, "read", &user).await?;

            Ok(Response::new(protobuf::GetUserReply { user: Some(user) }))
        }
        .instrument(debug_span!("get_user", ?request))
        .await
    }

    async fn get_organization(
        &self,
        request: Request<protobuf::GetOrganizationRequest>,
    ) -> TonicResult<protobuf::GetOrganizationReply> {
        async {
            debug!("get_organization");
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let organization: organization::Organization = self
                .db
                .get(req.organization_id.to_string())
                .await
                .map_err(|e| {
                    debug!(?e, "organization_get_failed");
                    AccountError::OrganizationMissing
                })?;
            debug!(?organization, "found");

            authorize(&self.db, user_id, billing_account_id, "read", &organization).await?;

            Ok(Response::new(protobuf::GetOrganizationReply {
                organization: Some(organization),
            }))
        }
        .instrument(debug_span!("get_organization", ?request))
        .await
    }

    async fn list_integration_instances(
        &self,
        request: Request<protobuf::ListIntegrationInstancesRequest>,
    ) -> TonicResult<protobuf::ListIntegrationInstancesReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            let scope_by_tenant_id = if req.scope_by_tenant_id == "" {
                billing_account_id
            } else {
                &req.scope_by_tenant_id
            };

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_integration_instances",
                &billing_account,
            )
            .await?;

            let list_result: ListResult<integration_instance::IntegrationInstance> =
                if req.page_token != "" {
                    self.db
                        .list_by_page_token(&req.page_token)
                        .await
                        .map_err(AccountError::ListIntegrationInstancesError)?
                } else {
                    self.db
                        .list(
                            &req.query,
                            req.page_size,
                            &req.order_by,
                            req.order_by_direction,
                            scope_by_tenant_id,
                            "",
                        )
                        .await
                        .map_err(AccountError::ListIntegrationInstancesError)?
                };

            if list_result.items.len() == 0 {
                return Ok(Response::new(
                    protobuf::ListIntegrationInstancesReply::default(),
                ));
            }

            Ok(Response::new(protobuf::ListIntegrationInstancesReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_integration_instances", ?request))
        .await
    }

    async fn list_integration_services(
        &self,
        request: Request<protobuf::ListIntegrationServicesRequest>,
    ) -> TonicResult<protobuf::ListIntegrationServicesReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            let scope_by_tenant_id = if req.scope_by_tenant_id == "" {
                "global"
            } else {
                &req.scope_by_tenant_id
            };

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_integrations",
                &billing_account,
            )
            .await?;

            let list_result: ListResult<integration::IntegrationService> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(AccountError::ListIntegrationServicesError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        scope_by_tenant_id,
                        "",
                    )
                    .await
                    .map_err(AccountError::ListIntegrationServicesError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(
                    protobuf::ListIntegrationServicesReply::default(),
                ));
            }

            Ok(Response::new(protobuf::ListIntegrationServicesReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_integration_services", ?request))
        .await
    }

    async fn list_integrations(
        &self,
        request: Request<protobuf::ListIntegrationsRequest>,
    ) -> TonicResult<protobuf::ListIntegrationsReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_integrations",
                &billing_account,
            )
            .await?;

            let list_result: ListResult<integration::Integration> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(AccountError::ListIntegrationsError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        "global",
                        "",
                    )
                    .await
                    .map_err(AccountError::ListIntegrationsError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListIntegrationsReply::default()));
            }

            Ok(Response::new(protobuf::ListIntegrationsReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_integrations", ?request))
        .await
    }

    async fn list_organizations(
        &self,
        request: Request<protobuf::ListOrganizationsRequest>,
    ) -> TonicResult<protobuf::ListOrganizationsReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_organizations",
                &billing_account,
            )
            .await?;

            let list_result: ListResult<organization::Organization> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(AccountError::ListOrganizationsError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        billing_account_id,
                        "",
                    )
                    .await
                    .map_err(AccountError::ListOrganizationsError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListOrganizationsReply::default()));
            }

            Ok(Response::new(protobuf::ListOrganizationsReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_organizations", ?request))
        .await
    }

    async fn list_workspaces(
        &self,
        request: Request<protobuf::ListWorkspacesRequest>,
    ) -> TonicResult<protobuf::ListWorkspacesReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let scope_by_tenant_id = if req.scope_by_tenant_id == "" {
                billing_account_id
            } else {
                &req.scope_by_tenant_id
            };

            authorize_by_tenant_id(&self.db, user_id, scope_by_tenant_id, "list_workspaces")
                .await?;

            let list_result: ListResult<workspace::Workspace> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(AccountError::ListWorkspacesError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        scope_by_tenant_id,
                        "",
                    )
                    .await
                    .map_err(AccountError::ListWorkspacesError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListWorkspacesReply::default()));
            }

            Ok(Response::new(protobuf::ListWorkspacesReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_workspaces", ?request))
        .await
    }

    async fn list_users(
        &self,
        request: Request<protobuf::ListUsersRequest>,
    ) -> TonicResult<protobuf::ListUsersReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            // You get authorization for listing contained items on
            // the containing item. I just made that up. It means you
            // certainly won't be able to, say, decide you can't see
            // individual things. I can imagine how to do that, but...
            // today is not the day.
            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "list_users",
                &billing_account,
            )
            .await?;

            let list_result: ListResult<user::User> = if req.page_token != "" {
                self.db
                    .list_by_page_token(&req.page_token)
                    .await
                    .map_err(AccountError::ListUsersError)?
            } else {
                self.db
                    .list(
                        &req.query,
                        req.page_size,
                        &req.order_by,
                        req.order_by_direction,
                        billing_account_id,
                        "",
                    )
                    .await
                    .map_err(AccountError::ListUsersError)?
            };

            if list_result.items.len() == 0 {
                return Ok(Response::new(protobuf::ListUsersReply::default()));
            }

            Ok(Response::new(protobuf::ListUsersReply {
                total_count: list_result.total_count(),
                next_page_token: list_result.page_token().to_string(),
                items: list_result.items,
            }))
        }
        .instrument(debug_span!("list_users", ?request))
        .await
    }

    async fn get_billing_account(
        &self,
        request: Request<protobuf::GetBillingAccountRequest>,
    ) -> TonicResult<protobuf::GetBillingAccountReply> {
        async {
            debug!("get_billing_account");
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let req = request.get_ref();

            let billing_account: billing_account::BillingAccount = self
                .db
                .get(req.billing_account_id.to_string())
                .await
                .map_err(|e| {
                    debug!(?e, "billing_account_get_failed");
                    AccountError::BillingAccountMissing
                })?;
            debug!(?billing_account, "found");

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "read",
                &billing_account,
            )
            .await?;

            Ok(Response::new(protobuf::GetBillingAccountReply {
                billing_account: Some(billing_account),
            }))
        }
        .instrument(debug_span!("get_billing_account", ?request))
        .await
    }

    // Login always returns a response, and it is purposefully vague
    // about why it succeeded or failed. This is to avoid information
    // leakage during brute-force attacks.
    async fn login(
        &self,
        request: Request<protobuf::LoginRequest>,
    ) -> TonicResult<protobuf::LoginReply> {
        async {
            debug!("login");
            let req = request.get_ref();

            let ba: billing_account::BillingAccount = match self
                .db
                .lookup(
                    "global",
                    billing_account::BillingAccount::type_name(),
                    &req.billing_account_short_name,
                )
                .await
            {
                Ok(ba) => ba,
                Err(err) => {
                    debug!(?err, "billing_account_lookup_failed");
                    return Ok(Response::new(protobuf::LoginReply {
                        authenticated: false,
                        ..Default::default()
                    }));
                }
            };

            let user: user::User = match self
                .db
                .lookup(
                    ba.id,
                    user::User::type_name().to_string(),
                    req.email.clone(),
                )
                .await
            {
                Ok(user) => user,
                Err(err) => {
                    debug!(?err, "login_lookup_failed");
                    return Ok(Response::new(protobuf::LoginReply {
                        authenticated: false,
                        ..Default::default()
                    }));
                }
            };

            let check_password = user.verify_password(&req.password);
            if check_password == true {
                debug!("login_check_succeeded");
                Ok(Response::new(protobuf::LoginReply {
                    authenticated: check_password,
                    billing_account_id: user.billing_account_id.clone(),
                    user_id: user.id,
                }))
            } else {
                debug!("login_check_failed");
                Ok(Response::new(protobuf::LoginReply {
                    authenticated: check_password,
                    ..Default::default()
                }))
            }
        }
        .instrument(debug_span!("login", ?request))
        .await
    }

    async fn create_integration_instance(
        &self,
        request: Request<protobuf::CreateIntegrationInstanceRequest>,
    ) -> TonicResult<protobuf::CreateIntegrationInstanceReply> {
        async {
            let metadata = request.metadata();
            let user_id = metadata
                .get("userId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;
            let billing_account_id = metadata
                .get("billingAccountId")
                .ok_or(AccountError::InvalidAuthentication)?
                .to_str()
                .map_err(AccountError::GrpcHeaderToString)?;

            // We authorize on the billing account, so we have to go get it.
            let billing_account: billing_account::BillingAccount = self
                .db
                .get(billing_account_id)
                .await
                .map_err(|_| AccountError::BillingAccountMissing)?;

            authorize(
                &self.db,
                user_id,
                billing_account_id,
                "create_integration_instance",
                &billing_account,
            )
            .await?;

            let req = request.get_ref();

            // These are only here because we are skipping the part where you
            // enable/disable for workspaces and organizations. There are only
            // the default ones, so every object is just going to be enabled
            // for them by default, for now.
            let organization: organization::Organization = self
                .db
                .lookup(billing_account_id, "organization", "default")
                .await
                .map_err(|_| AccountError::OrganizationMissing)?;

            let workspace: workspace::Workspace = self
                .db
                .lookup_by_natural_key(format!(
                    "{}:{}:workspace:default",
                    billing_account_id,
                    organization.get_id()
                ))
                .await
                .map_err(|_| AccountError::WorkspaceMissing)?;

            let mut integration_instance =
                integration_instance::IntegrationInstance::new_from_create_request(&req)?;

            integration_instance.add_to_tenant_ids(billing_account_id.to_string());
            integration_instance.billing_account_id = billing_account_id.to_string();
            integration_instance.enabled_on_workspace_ids = vec![workspace.get_id().to_string()];
            integration_instance.enabled_on_organization_ids =
                vec![organization.get_id().to_string()];

            self.db
                .validate_and_insert_as_new(&mut integration_instance)
                .await
                .map_err(AccountError::CreateIntegrationInstanceError)?;

            Ok(Response::new(protobuf::CreateIntegrationInstanceReply {
                integration_instance: Some(integration_instance),
            }))
        }
        .instrument(debug_span!("create_integration_instance", ?request))
        .await
    }

    async fn create_account(
        &self,
        request: Request<protobuf::CreateAccountRequest>,
    ) -> TonicResult<protobuf::CreateAccountReply> {
        async {
            let req = request.get_ref();

            // Only let users with @systeminit.com register
            if std::env::var_os("NO_SIGNUPS").is_some() {
                match &req.user {
                    Some(user) => {
                        if !user.email.contains("@systeminit.com") {
                            return Err(tonic::Status::from(AccountError::Authorization));
                        }
                    }
                    None => return Err(tonic::Status::from(AccountError::EmptyUser)),
                }
            }

            let mut ba = match &req.billing_account {
                Some(ba) => billing_account::BillingAccount::new_from_create_request(ba)?,
                None => return Err(tonic::Status::from(AccountError::EmptyBillingAccount)),
            };

            ba.add_to_tenant_ids("global".to_string());

            let mut user = match &req.user {
                Some(user) => user::User::new_from_create_request(user)?,
                None => return Err(tonic::Status::from(AccountError::EmptyUser)),
            };

            self.db
                .validate_and_insert_as_new(&mut ba)
                .await
                .map_err(AccountError::CreateBillingAccountError)?;

            user.billing_account_id = ba.id.clone();
            user.add_to_tenant_ids(ba.id.clone());

            self.db
                .validate_and_insert_as_new(&mut user)
                .await
                .map_err(AccountError::CreateUserError)?;

            let mut admin_group =
                group::Group::new("administrators".to_string(), "Administrators".to_string());
            admin_group.add_to_tenant_ids(ba.id.clone());
            admin_group.add_user(user.id.clone());
            admin_group.add_capability(ba.id.clone(), vec!["any".to_string()]);
            admin_group.billing_account_id = ba.id.clone();
            self.db
                .validate_and_insert_as_new(&mut admin_group)
                .await
                .map_err(AccountError::CreateGroupError)?;

            let mut organization =
                organization::Organization::new("default".to_string(), ba.id.clone());
            organization.add_to_tenant_ids(ba.id.clone());
            self.db
                .validate_and_insert_as_new(&mut organization)
                .await
                .map_err(AccountError::CreateOrganizationError)?;

            let mut workspace = workspace::Workspace::new(
                "default".to_string(),
                ba.id.clone(),
                organization.id.clone(),
            );
            workspace.add_to_tenant_ids(ba.id.clone());
            workspace.add_to_tenant_ids(organization.id.clone());
            self.db
                .validate_and_insert_as_new(&mut workspace)
                .await
                .map_err(AccountError::CreateWorkspaceError)?;

            let global_integration: integration::Integration = self
                .db
                .lookup_by_natural_key("global:integration:global")
                .await
                .map_err(|_| AccountError::IntegrationMissing)?;

            let mut integration_instance = integration_instance::IntegrationInstance {
                name: "global".to_string(),
                display_name: "Global Integration".to_string(),
                tenant_ids: vec![ba.get_id().to_string()],
                integration_id: global_integration.get_id().to_string(),
                integration_option_values: Vec::new(),
                billing_account_id: ba.get_id().to_string(),
                enabled_on_workspace_ids: vec![workspace.get_id().to_string()],
                enabled_on_organization_ids: vec![organization.get_id().to_string()],
                ..Default::default()
            };

            self.db
                .validate_and_insert_as_new(&mut integration_instance)
                .await
                .map_err(AccountError::CreateIntegrationInstanceError)?;

            let aws_integration: integration::Integration = self
                .db
                .lookup_by_natural_key("global:integration:aws")
                .await
                .map_err(|_| AccountError::IntegrationMissing)?;

            let mut aws_integration_instance = integration_instance::IntegrationInstance {
                name: "aws".to_string(),
                display_name: "AWS".to_string(),
                tenant_ids: vec![ba.get_id().to_string()],
                integration_id: aws_integration.get_id().to_string(),
                integration_option_values: Vec::new(),
                billing_account_id: ba.get_id().to_string(),
                enabled_on_workspace_ids: vec![workspace.get_id().to_string()],
                enabled_on_organization_ids: vec![organization.get_id().to_string()],
                ..Default::default()
            };

            self.db
                .validate_and_insert_as_new(&mut aws_integration_instance)
                .await
                .map_err(AccountError::CreateIntegrationInstanceError)?;

            Ok(Response::new(protobuf::CreateAccountReply {
                user: Some(user),
                billing_account: Some(ba),
                ..Default::default()
            }))
        }
        .instrument(debug_span!("create_account", ?request))
        .await
    }
}
