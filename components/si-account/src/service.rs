use si_data::{Db, ListResult, Storable};
use tonic::{Request, Response};
use tracing::{debug, debug_span};
use tracing_futures::Instrument;

use crate::authorize::{authorize, authorize_by_tenant_id};
use crate::error::{AccountError, TonicResult};
use crate::model::{billing_account, group, integration, organization, user, workspace};
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
