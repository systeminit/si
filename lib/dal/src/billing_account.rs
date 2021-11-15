use crate::standard_model::option_object_from_row;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_has_many,
    Capability, CapabilityError, Group, GroupError, HistoryActor, HistoryEventError, KeyPair,
    KeyPairError, Organization, OrganizationError, StandardModel, StandardModelError, Tenancy,
    Timestamp, User, UserError, Visibility, Workspace, WorkspaceError,
};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use thiserror::Error;

const BILLING_ACCOUNT_GET_BY_NAME: &str = include_str!("./queries/billing_account_get_by_name.sql");
const BILLING_ACCOUNT_GET_DEFAULTS: &str =
    include_str!("./queries/billing_account_get_defaults.sql");

#[derive(Error, Debug)]
pub enum BillingAccountError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("key pair error: {0}")]
    KeyPair(#[from] KeyPairError),
    #[error("user error: {0}")]
    User(#[from] UserError),
    #[error("group error: {0}")]
    Group(#[from] GroupError),
    #[error("capability error: {0}")]
    Capability(#[from] CapabilityError),
    #[error("organization error: {0}")]
    Organization(#[from] OrganizationError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
}

pub type BillingAccountResult<T> = Result<T, BillingAccountError>;

pk!(BillingAccountPk);
pk!(BillingAccountId);

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccount {
    pk: BillingAccountPk,
    id: BillingAccountId,
    name: String,
    description: Option<String>,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: BillingAccount,
    pk: BillingAccountPk,
    id: BillingAccountId,
    table_name: "billing_accounts",
    history_event_label_base: "billing_account",
    history_event_message_name: "Billing Account"
}

impl BillingAccount {
    #[tracing::instrument(skip(txn, nats, name, description))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        name: impl AsRef<str>,
        description: Option<&String>,
    ) -> BillingAccountResult<Self> {
        let name = name.as_ref();
        let row = txn
            .query_one(
                "SELECT object FROM billing_account_create_v1($1, $2, $3, $4)",
                &[&tenancy, &visibility, &name, &description],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            &txn,
            &nats,
            &tenancy,
            &visibility,
            &history_actor,
            row,
        )
        .await?;
        Ok(object)
    }

    standard_model_accessor!(name, String, BillingAccountResult);
    standard_model_accessor!(description, Option<String>, BillingAccountResult);

    standard_model_has_many!(
        lookup_fn: key_pairs,
        table: "key_pair_belongs_to_billing_account",
        model_table: "key_pairs",
        returns: KeyPair,
        result: BillingAccountResult,
    );

    standard_model_has_many!(
        lookup_fn: users,
        table: "user_belongs_to_billing_account",
        model_table: "users",
        returns: User,
        result: BillingAccountResult,
    );

    standard_model_has_many!(
        lookup_fn: groups,
        table: "group_belongs_to_billing_account",
        model_table: "groups",
        returns: Group,
        result: BillingAccountResult,
    );

    pub async fn signup(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        billing_account_name: impl AsRef<str>,
        user_name: impl AsRef<str>,
        user_email: impl AsRef<str>,
        user_password: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccountSignup> {
        let billing_account = BillingAccount::new(
            &txn,
            &nats,
            &tenancy,
            &visibility,
            &history_actor,
            &billing_account_name,
            None,
        )
        .await?;

        let billing_account_tenancy = Tenancy::new_billing_account(vec![*billing_account.id()]);

        let key_pair = KeyPair::new(
            &txn,
            &nats,
            &billing_account_tenancy,
            &visibility,
            &history_actor,
            "default",
        )
        .await?;
        key_pair
            .set_billing_account(&txn, &nats, &history_actor, billing_account.id())
            .await?;

        let user = User::new(
            &txn,
            &nats,
            &billing_account_tenancy,
            &visibility,
            &history_actor,
            &user_name,
            &user_email,
            &user_password,
        )
        .await?;
        user.set_billing_account(&txn, &nats, &history_actor, billing_account.id())
            .await?;

        let user_history_actor = HistoryActor::User(*user.id());

        let admin_group = Group::new(
            &txn,
            &nats,
            &billing_account_tenancy,
            &visibility,
            &user_history_actor,
            "administrators",
        )
        .await?;
        admin_group
            .set_billing_account(&txn, &nats, &user_history_actor, billing_account.id())
            .await?;
        admin_group
            .add_user(&txn, &nats, &user_history_actor, user.id())
            .await?;

        let any_cap = Capability::new(
            &txn,
            &nats,
            &billing_account_tenancy,
            &visibility,
            &user_history_actor,
            "any",
            "any",
        )
        .await?;
        any_cap
            .set_group(&txn, &nats, &user_history_actor, admin_group.id())
            .await?;

        let organization = Organization::new(
            &txn,
            &nats,
            &billing_account_tenancy,
            &visibility,
            &user_history_actor,
            "default",
        )
        .await?;
        organization
            .set_billing_account(&txn, &nats, &user_history_actor, billing_account.id())
            .await?;

        let organization_tenancy = Tenancy::new(
            false,
            billing_account_tenancy.billing_account_ids.clone(),
            vec![*organization.id()],
            vec![],
        );

        let workspace = Workspace::new(
            &txn,
            &nats,
            &organization_tenancy,
            &visibility,
            &user_history_actor,
            "default",
        )
        .await?;
        workspace
            .set_organization(&txn, &nats, &user_history_actor, organization.id())
            .await?;

        Ok(BillingAccountSignup {
            billing_account,
            key_pair,
            user,
            admin_group,
            organization,
            workspace,
        })
    }

    pub async fn find_by_name(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        name: impl AsRef<str>,
    ) -> BillingAccountResult<Option<BillingAccount>> {
        let name = name.as_ref();
        let maybe_row = txn
            .query_opt(BILLING_ACCOUNT_GET_BY_NAME, &[&name, &tenancy, &visibility])
            .await?;
        let result = option_object_from_row(maybe_row)?;
        Ok(result)
    }

    pub async fn get_defaults(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &BillingAccountId,
    ) -> BillingAccountResult<BillingAccountDefaults> {
        let row = txn
            .query_one(BILLING_ACCOUNT_GET_DEFAULTS, &[&tenancy, &visibility, &id])
            .await?;
        let organization_json: serde_json::Value = row.try_get("organization")?;
        let organization: Organization = serde_json::from_value(organization_json)?;
        let workspace_json: serde_json::Value = row.try_get("workspace")?;
        let workspace: Workspace = serde_json::from_value(workspace_json)?;
        let result = BillingAccountDefaults {
            organization,
            workspace,
        };
        Ok(result)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccountSignup {
    pub billing_account: BillingAccount,
    pub key_pair: KeyPair,
    pub user: User,
    pub admin_group: Group,
    pub organization: Organization,
    pub workspace: Workspace,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccountDefaults {
    pub organization: Organization,
    pub workspace: Workspace,
}
