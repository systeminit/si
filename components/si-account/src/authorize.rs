use serde_json::json;
use si_data::{Db, Storable};
use tracing::{debug, info};

use std::collections::HashMap;

use crate::error::{AccountError, Result};
use crate::model::group::Group;
use crate::model::user::User;
use crate::model::{billing_account, organization, workspace};

pub async fn authorize_by_tenant_id(
    db: &Db,
    actor_id: &str,
    tenant_id: &str,
    action: &str,
) -> Result<()> {
    if tenant_id.starts_with("billing_account:") {
        let t: billing_account::BillingAccount = db
            .get(tenant_id)
            .await
            .map_err(AccountError::UnknownTenantId)?;
        return authorize(db, actor_id, tenant_id, action, &t).await;
    } else if tenant_id.starts_with("organization:") {
        let t: organization::Organization = db
            .get(tenant_id)
            .await
            .map_err(AccountError::UnknownTenantId)?;
        return authorize(db, actor_id, tenant_id, action, &t).await;
    } else if tenant_id.starts_with("workspace:") {
        let t: workspace::Workspace = db
            .get(tenant_id)
            .await
            .map_err(AccountError::UnknownTenantId)?;
        return authorize(db, actor_id, tenant_id, action, &t).await;
    }
    Err(AccountError::InvalidTenantId)
}

#[tracing::instrument]
pub async fn authorize<T: Storable + std::fmt::Debug>(
    db: &Db,
    actor_id: &str,
    tenant_id: &str,
    action: &str,
    subject: &T,
) -> Result<()> {
    debug!("authorize_get_user");
    // For now, the actor is always a user. In theory, we can use this
    // same thing to deal with any actor. Just not today.
    let user: User = db.get(actor_id).await?;
    debug!(?user);

    // If the actor has a capability that matches any object in the subjects
    // tenant_id list, and the allowed actions are either the one we asked for
    // or "any", then it is authorized.
    for capability in user.capabilities.iter() {
        if subject
            .get_tenant_ids()
            .iter()
            .any(|tenant_id| *tenant_id == capability.subject)
            && capability.actions.iter().any(|a| a == action || a == "any")
        {
            debug!(authorized = true);
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
    named_params.insert("tenant_id".into(), json![user.billing_account_id]);
    named_params.insert("actor_id".into(), json![actor_id]);
    named_params.insert("subject_id".into(), json![subject.get_id()]);

    let query = format!("SELECT {bucket}.* FROM {bucket} WHERE typeName = \"group\" AND ARRAY_CONTAINS(tenantIds, $tenant_id) AND ARRAY_CONTAINS(userIds, $actor_id)", bucket=db.bucket_name);

    let groups: Vec<Group> = db.query(query, Some(named_params)).await?;

    for group in groups.iter() {
        for capability in group.capabilities.iter() {
            if subject
                .get_tenant_ids()
                .iter()
                .any(|tenant_id| *tenant_id == capability.subject)
                && capability.actions.iter().any(|a| a == action || a == "any")
            {
                debug!(?group, authorized = true);
                return Ok(());
            }
        }
    }

    info!(?user, ?groups, authorized = false, "authorize_failed");

    Err(AccountError::Authorization)
}
