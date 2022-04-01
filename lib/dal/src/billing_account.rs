use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    impl_standard_model, pk,
    schema::variant::SchemaVariantError,
    standard_model::{self, option_object_from_row},
    standard_model_accessor, standard_model_has_many, Capability, CapabilityError, DalContext,
    Group, GroupError, HistoryActor, HistoryEventError, KeyPair, KeyPairError, NodeError,
    Organization, OrganizationError, ReadTenancy, ReadTenancyError, SchemaError, StandardModel,
    StandardModelError, System, SystemError, Timestamp, User, UserError, Visibility, Workspace,
    WorkspaceError, WriteTenancy,
};

const INITIAL_SYSTEM_NAME: &str = "production";
const BILLING_ACCOUNT_GET_BY_NAME: &str = include_str!("./queries/billing_account_get_by_name.sql");
const BILLING_ACCOUNT_GET_DEFAULTS: &str =
    include_str!("./queries/billing_account_get_defaults.sql");

#[derive(Error, Debug)]
pub enum BillingAccountError {
    #[error("initial system missing for get_defaults")]
    InitialSystemMissing,
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
    #[error("node error: {0}")]
    Node(#[from] NodeError),
    #[error("organization error: {0}")]
    Organization(#[from] OrganizationError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("system error: {0}")]
    System(#[from] SystemError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
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
    tenancy: WriteTenancy,
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
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        description: Option<&String>,
    ) -> BillingAccountResult<Self> {
        let name = name.as_ref();
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM billing_account_create_v1($1, $2, $3, $4)",
                &[ctx.write_tenancy(), ctx.visibility(), &name, &description],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
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
        ctx: &DalContext<'_, '_>,
        billing_account_name: impl AsRef<str>,
        user_name: impl AsRef<str>,
        user_email: impl AsRef<str>,
        user_password: impl AsRef<str>,
    ) -> BillingAccountResult<BillingAccountSignup> {
        let billing_account = BillingAccount::new(ctx, billing_account_name, None).await?;

        let billing_account_tenancy = WriteTenancy::new_billing_account(*billing_account.id());
        let mut ctx = ctx.clone_with_new_write_tenancy(billing_account_tenancy);

        let key_pair = KeyPair::new(&ctx, "default").await?;
        key_pair
            .set_billing_account(&ctx, billing_account.id())
            .await?;

        let user = User::new(&ctx, &user_name, &user_email, &user_password).await?;
        user.set_billing_account(&ctx, billing_account.id()).await?;
        let user_history_actor = HistoryActor::User(*user.id());
        ctx.update_history_actor(user_history_actor);

        // TODO: remove the bobo user before we ship!
        let user_bobo = User::new(
            &ctx,
            &user_name,
            &format!("bobo-{}", user_email.as_ref()),
            &user_password,
        )
        .await?;
        user_bobo
            .set_billing_account(&ctx, billing_account.id())
            .await?;

        let admin_group = Group::new(&ctx, "administrators").await?;
        admin_group
            .set_billing_account(&ctx, billing_account.id())
            .await?;
        admin_group.add_user(&ctx, user.id()).await?;
        admin_group.add_user(&ctx, user_bobo.id()).await?;

        let any_cap = Capability::new(&ctx, "any", "any").await?;
        any_cap.set_group(&ctx, admin_group.id()).await?;

        let organization = Organization::new(&ctx, "default").await?;
        organization
            .set_billing_account(&ctx, billing_account.id())
            .await?;

        let organization_read_tenancy = ReadTenancy::new_organization(
            ctx.txns().pg(),
            vec![*organization.id()],
            ctx.visibility(),
        )
        .await?;
        let organization_write_tenancy: WriteTenancy =
            WriteTenancy::new_organization(*organization.id());
        ctx.update_read_tenancy(organization_read_tenancy);
        ctx.update_write_tenancy(organization_write_tenancy);

        let workspace = Workspace::new(&ctx, "default").await?;
        workspace.set_organization(&ctx, organization.id()).await?;

        let workspace_read_tenancy =
            ReadTenancy::new_workspace(ctx.txns().pg(), vec![*workspace.id()], ctx.visibility())
                .await?;
        let workspace_write_tenancy = WriteTenancy::new_workspace(*workspace.id());
        ctx.update_read_tenancy(workspace_read_tenancy);
        ctx.update_write_tenancy(workspace_write_tenancy);

        let (system, _system_node) = System::new_with_node(&ctx, INITIAL_SYSTEM_NAME).await?;
        system.set_workspace(&ctx, workspace.id()).await?;

        Ok(BillingAccountSignup {
            billing_account,
            key_pair,
            user,
            admin_group,
            organization,
            workspace,
            system,
        })
    }

    pub async fn find_by_name(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
    ) -> BillingAccountResult<Option<BillingAccount>> {
        let name = name.as_ref();
        let maybe_row = ctx
            .txns()
            .pg()
            .query_opt(
                BILLING_ACCOUNT_GET_BY_NAME,
                &[&name, ctx.read_tenancy(), ctx.visibility()],
            )
            .await?;
        let result = option_object_from_row(maybe_row)?;
        Ok(result)
    }

    pub async fn get_defaults(
        ctx: &DalContext<'_, '_>,
        id: &BillingAccountId,
    ) -> BillingAccountResult<BillingAccountDefaults> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                BILLING_ACCOUNT_GET_DEFAULTS,
                &[ctx.read_tenancy(), ctx.visibility(), &id],
            )
            .await?;
        let organization_json: serde_json::Value = row.try_get("organization")?;
        let organization: Organization = serde_json::from_value(organization_json)?;
        let workspace_json: serde_json::Value = row.try_get("workspace")?;
        let workspace: Workspace = serde_json::from_value(workspace_json)?;

        let mut workspace_ctx = ctx.clone();
        let read_tenancy =
            ReadTenancy::new_workspace(ctx.txns().pg(), vec![*workspace.id()], ctx.visibility())
                .await?;
        workspace_ctx.update_read_tenancy(read_tenancy);

        // TODO(fnichol): this query should get rolled up into the above query...
        let system = workspace
            .systems(&workspace_ctx)
            .await?
            .into_iter()
            .find(|system| system.name() == INITIAL_SYSTEM_NAME)
            .ok_or(BillingAccountError::InitialSystemMissing)?;

        let result = BillingAccountDefaults {
            organization,
            workspace,
            system,
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
    pub system: System,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BillingAccountDefaults {
    pub organization: Organization,
    pub workspace: Workspace,
    pub system: System,
}
