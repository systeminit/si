use serde::{Deserialize, Serialize};
use thiserror::Error;

use std::collections::HashMap;

use crate::data::{Connection, Db};
use crate::models::{
    key_pair::KeyPair,
    {
        check_secondary_key_universal, generate_id, get_model, insert_model, upsert_model,
        Capability, Group, GroupError, KeyPairError, ModelError, Organization, OrganizationError,
        SiStorableError, SimpleStorable, User, UserError, Workspace, WorkspaceError,
    },
};

#[derive(Error, Debug)]
pub enum BillingAccountError {
    #[error("a billing account with this name already exists")]
    AccountExists,
    #[error("si_storable error: {0}")]
    SiStorable(#[from] SiStorableError),
    #[error("error in core model functions: {0}")]
    Model(#[from] ModelError),
    #[error("error in user model: {0}")]
    User(#[from] UserError),
    #[error("error in group model: {0}")]
    Group(#[from] GroupError),
    #[error("error in organization model: {0}")]
    Organization(#[from] OrganizationError),
    #[error("error in key pair model: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("error in workspace model: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("database error: {0}")]
    Data(#[from] crate::data::DataError),
    #[error("billing account is not found")]
    NotFound,
}

pub type BillingAccountResult<T> = Result<T, BillingAccountError>;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub billing_account_name: String,
    pub billing_account_description: String,
    pub user_name: String,
    pub user_email: String,
    pub user_password: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateReply {
    pub billing_account: BillingAccount,
    pub user: User,
    pub group: Group,
    pub organization: Organization,
    pub workspace: Workspace,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BillingAccount {
    pub id: String,
    pub name: String,
    pub description: String,
    pub current_key_pair_id: String,
    pub si_storable: SimpleStorable,
}

impl BillingAccount {
    pub async fn new(
        db: &Db,
        nats: &Connection,
        name: String,
        description: String,
    ) -> BillingAccountResult<Self> {
        if check_secondary_key_universal(db, "billingAccount", "name", &name).await? {
            return Err(BillingAccountError::AccountExists);
        }
        let id = generate_id("billingAccount");
        let si_storable = SimpleStorable::new(&id, "billingAccount", &id);
        let key_pair = KeyPair::new(db, nats, &name, id.clone()).await?;
        let current_key_pair_id = key_pair.id;
        let model = Self {
            id,
            name,
            description,
            current_key_pair_id,
            si_storable,
        };
        insert_model(db, nats, &model.id, &model).await?;
        Ok(model)
    }

    pub async fn signup(
        db: &Db,
        nats: &Connection,
        billing_account_name: String,
        billing_account_description: String,
        user_name: String,
        user_email: String,
        user_password: String,
    ) -> BillingAccountResult<(BillingAccount, User, Group, Organization, Workspace)> {
        let billing_account =
            BillingAccount::new(db, nats, billing_account_name, billing_account_description)
                .await?;

        let user = User::new(
            db,
            nats,
            user_name,
            user_email,
            &billing_account.id,
            user_password,
        )
        .await?;

        let admin_group = Group::new(
            db,
            nats,
            "administrators",
            vec![user.id.clone()],
            vec![Capability::new("any", "any")],
            &billing_account.id,
        )
        .await?;

        let organization = Organization::new(db, nats, "default", &billing_account.id).await?;

        let workspace = Workspace::new(db, nats, "default", &billing_account.id).await?;

        Ok((billing_account, user, admin_group, organization, workspace))
    }

    pub async fn get(
        db: &Db,
        billing_account_id: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccount> {
        let id = billing_account_id.as_ref();
        let object: BillingAccount = get_model(db, id, id).await?;
        Ok(object)
    }

    pub async fn get_by_name(
        db: &Db,
        billing_account_name: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccount> {
        let billing_account_name = billing_account_name.as_ref();

        let query = format!(
            "SELECT a.*
               FROM `{bucket}` AS a
               WHERE a.siStorable.typeName = \"billingAccount\"
                 AND a.name = $billing_account_name
               LIMIT 1",
            bucket = db.bucket_name,
        );
        let mut named_params: HashMap<String, serde_json::Value> = HashMap::new();
        named_params.insert(
            "billing_account_name".into(),
            serde_json::json![billing_account_name],
        );
        let mut results: Vec<BillingAccount> = db.query(query, Some(named_params)).await?;
        if let Some(billing_account) = results.pop() {
            Ok(billing_account)
        } else {
            Err(BillingAccountError::NotFound)
        }
    }

    pub async fn rotate_key_pair(
        db: &Db,
        nats: &Connection,
        billing_account_id: impl AsRef<str>,
    ) -> BillingAccountResult<()> {
        let mut billing_account = Self::get(db, billing_account_id).await?;
        let new_key_pair =
            KeyPair::new(db, nats, &billing_account.name, &billing_account.id).await?;

        billing_account.current_key_pair_id = new_key_pair.id;
        upsert_model(db, nats, &billing_account.id, &billing_account).await?;

        Ok(())
    }
}
