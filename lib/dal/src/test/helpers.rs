use names::{Generator, Name};

use crate::{
    BillingAccount, BillingAccountId, BillingAccountSignup, ChangeSet, Component, DalContext,
    DalContextBuilder, EditSession, Group, HistoryActor, JwtSecretKey, RequestContext, Schema,
    SchemaId, SchemaVariant, StandardModel, System, Tenancy, Transactions, User, Visibility,
    WriteTenancy,
};

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn billing_account_signup(
    builder: &DalContextBuilder,
    txns: &Transactions<'_>,
    jwt_secret_key: &JwtSecretKey,
) -> (BillingAccountSignup, String) {
    let request_context = RequestContext::new_universal_head(HistoryActor::SystemInit);
    let ctx = builder.build(request_context, txns);

    let billing_account_name = generate_fake_name();
    let user_name = format!("frank {}", billing_account_name);
    let user_email = format!("{}@example.com", billing_account_name);
    let user_password = "snakes";

    let nba = BillingAccount::signup(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await
    .expect("cannot signup a new billing_account");
    let auth_token = nba
        .user
        .login(
            ctx.pg_txn(),
            jwt_secret_key,
            nba.billing_account.id(),
            "snakes",
        )
        .await
        .expect("cannot log in newly created user");
    (nba, auth_token)
}

pub async fn create_group(ctx: &DalContext<'_, '_>) -> Group {
    let name = generate_fake_name();
    Group::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        &name,
    )
    .await
    .expect("cannot create group")
}

pub async fn create_user(ctx: &DalContext<'_, '_>) -> User {
    let name = generate_fake_name();
    User::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        &name,
        &format!("{}@test.systeminit.com", name),
        "liesAreTold",
    )
    .await
    .expect("cannot create user")
}

pub async fn create_billing_account_with_name(
    ctx: &DalContext<'_, '_>,
    name: impl AsRef<str>,
) -> BillingAccount {
    BillingAccount::new(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.write_tenancy(),
        ctx.visibility(),
        ctx.history_actor(),
        name,
        None,
    )
    .await
    .expect("cannot create billing_account")
}

pub async fn create_billing_account(ctx: &DalContext<'_, '_>) -> BillingAccount {
    let name = generate_fake_name();
    create_billing_account_with_name(ctx, name).await
}

pub async fn create_change_set(
    txns: &Transactions<'_>,
    history_actor: &HistoryActor,
    billing_account_id: BillingAccountId,
) -> ChangeSet {
    let write_tenancy = WriteTenancy::new_billing_account(billing_account_id);
    let name = generate_fake_name();
    ChangeSet::new(
        txns.pg(),
        txns.nats(),
        &write_tenancy,
        history_actor,
        &name,
        None,
    )
    .await
    .expect("cannot create change_set")
}

pub async fn create_edit_session(
    txns: &Transactions<'_>,
    history_actor: &HistoryActor,
    change_set: &ChangeSet,
) -> EditSession {
    let name = generate_fake_name();
    EditSession::new(
        txns.pg(),
        txns.nats(),
        &(&change_set.tenancy).into(),
        history_actor,
        &change_set.pk,
        &name,
        None,
    )
    .await
    .expect("cannot create edit_session")
}

pub async fn create_change_set_and_edit_session(
    txns: &Transactions<'_>,
    history_actor: &HistoryActor,
    billing_account_id: BillingAccountId,
) -> (ChangeSet, EditSession) {
    let change_set = create_change_set(txns, history_actor, billing_account_id).await;
    let edit_session = create_edit_session(txns, history_actor, &change_set).await;
    (change_set, edit_session)
}

pub fn create_visibility_for_change_set_and_edit_session(
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> Visibility {
    Visibility::new(change_set.pk, edit_session.pk, false)
}

/// Creates a new [`Visibility`] backed by a new [`ChangeSet`] and a new [`EditSession`].
pub async fn create_visibility_for_new_change_set_and_edit_session(
    txns: &Transactions<'_>,
    billing_account_id: BillingAccountId,
) -> Visibility {
    let history_actor = HistoryActor::SystemInit;
    let (change_set, edit_session) =
        create_change_set_and_edit_session(txns, &history_actor, billing_account_id).await;

    create_visibility_for_change_set_and_edit_session(&change_set, &edit_session)
}

/// Creates a new [`DalContext`] in a change set and edit session in the given billing account.
pub async fn create_ctx_for_new_change_set_and_edit_session<'s, 't>(
    builder: &'s DalContextBuilder,
    txns: &'t Transactions<'t>,
    nba: &BillingAccountSignup,
) -> DalContext<'s, 't> {
    let mut request_context = RequestContext::new_workspace_head(
        txns.pg(),
        HistoryActor::SystemInit,
        *nba.workspace.id(),
    )
    .await
    .expect("failed to create request context");
    let visibility =
        create_visibility_for_new_change_set_and_edit_session(txns, *nba.billing_account.id())
            .await;
    request_context.visibility = visibility;

    builder.build(request_context, txns)
}

pub async fn new_ctx_for_new_change_set_and_edit_session<'a, 'b>(
    ctx: &DalContext<'a, 'b>,
    billing_account_id: BillingAccountId,
) -> DalContext<'a, 'b> {
    let visibility =
        create_visibility_for_new_change_set_and_edit_session(ctx.txns(), billing_account_id).await;
    ctx.clone_with_new_visibility(visibility)
}

pub async fn create_component_for_schema(
    ctx: &DalContext<'_, '_>,
    schema_id: &SchemaId,
) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_schema_with_node(
        ctx.pg_txn(),
        ctx.nats_txn(),
        ctx.veritech().clone(),
        ctx.encryption_key(),
        &ctx.write_tenancy().into(),
        ctx.visibility(),
        ctx.history_actor(),
        &name,
        schema_id,
    )
    .await
    .expect("cannot create component");
    component
        .set_schema(
            ctx.pg_txn(),
            ctx.nats_txn(),
            ctx.visibility(),
            ctx.history_actor(),
            schema_id,
        )
        .await
        .expect("cannot set the schema for our component");
    component
}

pub async fn find_schema_by_name(ctx: &DalContext<'_, '_>, schema_name: impl AsRef<str>) -> Schema {
    Schema::find_by_attr(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        "name",
        &schema_name.as_ref(),
    )
    .await
    .expect("cannot find schema by name")
    .pop()
    .expect("no schema found")
}

pub async fn find_schema_and_default_variant_by_name(
    ctx: &DalContext<'_, '_>,
    schema_name: impl AsRef<str>,
) -> (Schema, SchemaVariant) {
    let schema = find_schema_by_name(ctx, schema_name).await;
    let default_variant = schema
        .default_variant(ctx.pg_txn(), ctx.read_tenancy(), ctx.visibility())
        .await
        .expect("cannot get default schema variant");
    (schema, default_variant)
}

pub async fn find_or_create_production_system(ctx: &DalContext<'_, '_>) -> System {
    let name = "production".to_string();

    match System::find_by_attr(
        ctx.pg_txn(),
        &Tenancy::from(ctx.read_tenancy()),
        ctx.visibility(),
        "name",
        &name,
    )
    .await
    .expect("cannot find system")
    .pop()
    {
        Some(s) => s,
        None => {
            let (system, _system_node) = System::new_with_node(
                ctx.pg_txn(),
                ctx.nats_txn(),
                ctx.write_tenancy(),
                ctx.visibility(),
                ctx.history_actor(),
                name,
            )
            .await
            .expect("cannot create named system");

            system
        }
    }
}
