use serde_json::json;
use si_data::{Db, Storable};
use tracing::{debug, info, info_span};
use tracing_futures::Instrument;

use std::collections::HashMap;

use crate::error::{AccountError, Result};
use crate::protobuf::{BillingAccount, Group, Organization, User, Workspace};

use std::convert::TryFrom;

#[derive(Debug)]
pub struct Authentication {
    user_id: String,
    billing_account_id: String,
}

impl Authentication {
    pub fn new(user_id: impl Into<String>, billing_account_id: impl Into<String>) -> Self {
        Authentication {
            user_id: user_id.into(),
            billing_account_id: billing_account_id.into(),
        }
    }

    pub fn billing_account_id(&self) -> &str {
        self.billing_account_id.as_ref()
    }

    pub fn user_id(&self) -> &str {
        self.user_id.as_ref()
    }

    pub async fn billing_account(&self, db: &Db) -> Result<BillingAccount> {
        let billing_account: BillingAccount = db.get(self.billing_account_id()).await?;
        Ok(billing_account)
    }

    pub async fn authorize_on_billing_account(
        &self,
        db: &Db,
        action: impl Into<&str>,
    ) -> Result<()> {
        let ba = self.billing_account(db).await?;
        authorize(
            db,
            self.user_id(),
            self.billing_account_id(),
            action.into(),
            &ba,
        )
        .await?;
        Ok(())
    }
}

impl<T> TryFrom<&tonic::Request<T>> for Authentication {
    type Error = AccountError;

    fn try_from(request: &tonic::Request<T>) -> std::result::Result<Self, Self::Error> {
        let metadata = request.metadata();
        let user_id = metadata
            .get("userId")
            .ok_or(AccountError::MissingField("userId".into()))?
            .to_str()?;
        let billing_account_id = metadata
            .get("billingAccountId")
            .ok_or(AccountError::MissingField("billingAccountId".into()))?
            .to_str()?;
        Ok(Authentication::new(user_id, billing_account_id))
    }
}

pub async fn authnz<T>(
    db: &Db,
    request: &tonic::Request<T>,
    endpoint: impl AsRef<str>,
) -> Result<Authentication>
where
    T: std::fmt::Debug,
{
    tracing::debug!(?request);
    let auth = Authentication::try_from(request)?;
    auth.authorize_on_billing_account(db, endpoint.as_ref())
        .await?;

    Ok(auth)
}

pub async fn authorize_by_tenant_id(
    db: &Db,
    actor_id: &str,
    tenant_id: &str,
    action: &str,
) -> Result<()> {
    if tenant_id.starts_with("billing_account:") {
        let t: BillingAccount = db
            .get(tenant_id)
            .await
            .map_err(AccountError::UnknownTenantId)?;
        return authorize(db, actor_id, tenant_id, action, &t).await;
    } else if tenant_id.starts_with("organization:") {
        let t: Organization = db
            .get(tenant_id)
            .await
            .map_err(AccountError::UnknownTenantId)?;
        return authorize(db, actor_id, tenant_id, action, &t).await;
    } else if tenant_id.starts_with("workspace:") {
        let t: Workspace = db
            .get(tenant_id)
            .await
            .map_err(AccountError::UnknownTenantId)?;
        return authorize(db, actor_id, tenant_id, action, &t).await;
    }
    Err(AccountError::InvalidTenantId)
}

pub async fn authorize<T: Storable + std::fmt::Debug>(
    db: &Db,
    actor_id: &str,
    _tenant_id: &str,
    action: &str,
    subject: &T,
) -> Result<()> {
    async {
        debug!("authorize_get_user");
        // For now, the actor is always a user. In theory, we can use this
        // same thing to deal with any actor. Just not today.
        let user: User = db.get(actor_id).await?;
        info!(?user);

        // If the actor has a capability that matches any object in the subjects
        // tenant_id list, and the allowed actions are either the one we asked for
        // or "any", then it is authorized.
        for capability in user.capabilities.iter() {
            if subject
                .get_tenant_ids()
                .iter()
                // Why FLOOPYBOODLES? Well, because capability's subject is now an option, and there
                // is no world where FLOOPYBOODLES is a valid tenant ID. So.. it lets us fail the
                // check without getting overly complicated. Of course, I probably could have just
                // done something else.. but who doesn't love a FLOOPYBOODLES?
                .any(|tenant_id| {
                    tenant_id
                        == capability
                            .subject
                            .as_ref()
                            .unwrap_or(&"FLOOPYBOODLES".into())
                })
                && capability.actions.iter().any(|a| a == action || a == "any")
            {
                info!(authorized = true);
                return Ok(());
            }
        }

        // If we didn't authorize off the user directly, then we check the
        // groups the user is a part of. This returns all groups that are
        // in our tenant id, have the user in the list of userids, and have
        // a capability matching the subjects id. We then check the results
        // to see if the requested action is allowed, or if the "any" action
        // is set.
        debug!("authorize_get_groups");
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert(
            "tenant_id".into(),
            json![user.si_properties.unwrap().billing_account_id],
        );
        named_params.insert("actor_id".into(), json![actor_id]);
        named_params.insert("subject_id".into(), json![subject.get_id()]);

        let query = format!("SELECT {bucket}.* FROM {bucket} WHERE typeName = \"group\" AND ARRAY_CONTAINS(tenantIds, $tenant_id) AND ARRAY_CONTAINS(userIds, $actor_id)", bucket=db.bucket_name);

        let groups: Vec<Group> = db.query(query, Some(named_params)).await?;

        for group in groups.iter() {
            for capability in group.capabilities.iter() {
                if subject.get_tenant_ids().iter().any(|tenant_id| {
                    tenant_id
                        == capability
                            .subject
                            .as_ref()
                            .unwrap_or(&"FLOOPYBOODLES".into())
                }) && capability.actions.iter().any(|a| a == action || a == "any")
                {
                    debug!(?group, authorized = true);
                    return Ok(());
                }
            }
        }

        info!(?groups, authorized = false, "authorize_failed");

        Err(AccountError::Authorization)
    }
    .instrument(info_span!("authorize"))
    .await
}
