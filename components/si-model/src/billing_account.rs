use crate::{
    system, Capability, ChangeSet, ChangeSetError, EditSession, EditSessionError, Group,
    GroupError, KeyPair, KeyPairError, Organization, OrganizationError, PublicKey, SimpleStorable,
    User, UserError, Workspace, WorkspaceError,
};
use crate::{SystemError, Veritech};
use serde::{Deserialize, Serialize};
use si_data::{pg::SqlState, NatsConn, NatsTxn, NatsTxnError, PgPool, PgTxn};
use thiserror::Error;

const BILLING_ACCOUNT_GET_BY_NAME: &str = include_str!("./queries/billing_account_get_by_name.sql");

#[derive(Error, Debug)]
pub enum BillingAccountError {
    #[error("a billing account with this name already exists")]
    AccountExists,
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("edit session error: {0}")]
    EditSession(#[from] EditSessionError),
    #[error("error in group model: {0}")]
    Group(#[from] GroupError),
    #[error("error in key pair model: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("nats txn error: {0}")]
    NatsTxn(#[from] NatsTxnError),
    #[error("billing account is not found")]
    NotFound,
    #[error("error in organization model: {0}")]
    Organization(#[from] OrganizationError),
    #[error("pg error: {0}")]
    Pg(#[from] si_data::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data::PgPoolError),
    #[error("serde error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("error in user model: {0}")]
    User(#[from] UserError),
    #[error("error in workspace model: {0}")]
    Workspace(#[from] WorkspaceError),
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
    //pub system: System,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BillingAccount {
    pub id: String,
    pub name: String,
    pub description: String,
    pub si_storable: SimpleStorable,
}

impl BillingAccount {
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> BillingAccountResult<Self> {
        let name = name.into();
        let description = description.into();
        let row = txn
            .query_one(
                "SELECT object FROM billing_account_create_v1($1, $2)",
                &[&name, &description],
            )
            .await
            .map_err(|err| match err.code() {
                Some(sql_state) if sql_state == &SqlState::UNIQUE_VIOLATION => {
                    BillingAccountError::AccountExists
                }
                _ => BillingAccountError::Pg(err),
            })?;
        let json: serde_json::Value = row.try_get("object")?;
        nats.publish(&json).await?;
        let object: BillingAccount = serde_json::from_value(json)?;

        Ok(object)
    }

    pub async fn signup(
        pg: &PgPool,
        txn: PgTxn<'_>,
        nats: &NatsTxn,
        nats_conn: &NatsConn,
        veritech: &Veritech,
        billing_account_name: impl Into<String>,
        billing_account_description: impl Into<String>,
        user_name: impl Into<String>,
        user_email: impl Into<String>,
        user_password: impl Into<String>,
    ) -> BillingAccountResult<(
        BillingAccount,
        User,
        Group,
        Organization,
        Workspace,
        PublicKey,
        //System,
    )> {
        let billing_account = BillingAccount::new(
            &txn,
            &nats,
            billing_account_name,
            billing_account_description,
        )
        .await?;

        let key_pair =
            KeyPair::new(&txn, &nats, &billing_account.name, &billing_account.id).await?;

        let user = User::new(
            &txn,
            &nats,
            user_name,
            user_email,
            user_password,
            &billing_account.id,
        )
        .await?;

        let admin_group = Group::new(
            &txn,
            &nats,
            "administrators",
            vec![user.id.clone()],
            vec![],
            vec![Capability::new("any", "any")],
            &billing_account.id,
        )
        .await?;

        let organization = Organization::new(&txn, &nats, "default", &billing_account.id).await?;

        let workspace = Workspace::new(
            &txn,
            &nats,
            "default",
            &billing_account.id,
            &organization.id,
        )
        .await?;

        txn.commit().await?;

        let mut cs_conn = pg.get().await?;
        let cs_txn = cs_conn.transaction().await?;
        let mut change_set = ChangeSet::new(&cs_txn, &nats, None, workspace.id.clone()).await?;
        let mut edit_session = EditSession::new(
            &cs_txn,
            &nats,
            None,
            change_set.id.clone(),
            workspace.id.clone(),
        )
        .await?;

        // TODO: This should be removed once you can create your own systems.
        let _system_entity = system::create(
            &pg,
            &cs_txn,
            &nats_conn,
            &nats,
            &veritech,
            Some("production".to_string()),
            &workspace.id,
            &change_set.id,
            &edit_session.id,
        )
        .await?;
        edit_session.save_session(&cs_txn).await?;
        change_set.apply(&cs_txn).await?;

        cs_txn.commit().await?;
        //let system = System::get_head(&system_txn, &system_node.object_id).await?;

        Ok((
            billing_account,
            user,
            admin_group,
            organization,
            workspace,
            key_pair.into(),
            //    system,
        ))
    }

    pub async fn get(
        txn: &PgTxn<'_>,
        billing_account_id: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccount> {
        let id = billing_account_id.as_ref();
        let row = txn
            .query_one("SELECT object FROM billing_account_get_v1($1)", &[&id])
            .await?;
        let json: serde_json::Value = row.try_get("object")?;
        let object = serde_json::from_value(json)?;
        Ok(object)
    }

    pub async fn get_by_name(
        txn: &PgTxn<'_>,
        name: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccount> {
        let name = name.as_ref();

        let row = txn.query_one(BILLING_ACCOUNT_GET_BY_NAME, &[&name]).await?;
        let json: serde_json::Value = row.try_get("object")?;
        let ba = serde_json::from_value(json)?;
        Ok(ba)
    }

    pub async fn rotate_key_pair(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        billing_account_id: impl AsRef<str>,
    ) -> BillingAccountResult<()> {
        let billing_account = Self::get(&txn, billing_account_id).await?;
        let _new_key_pair =
            KeyPair::new(&txn, &nats, &billing_account.name, &billing_account.id).await?;
        Ok(())
    }
}
