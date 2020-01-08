use si_data::{Db, Storable};
use tonic::{Request, Response};
use tracing::{debug, debug_span};
use tracing_futures::Instrument;

use crate::authorize::authorize;
use crate::error::{AccountError, TonicResult};
use crate::model::{billing_account, group, user};
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
            self.db
                .validate_and_insert_as_new(&mut admin_group)
                .await
                .map_err(AccountError::CreateGroupError)?;

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
