use color_eyre::Result;
use dal::{
    func::{
        argument::{FuncArgument, FuncArgumentId},
        binding::FuncBindingId,
        binding_return_value::FuncBindingReturnValueId,
    },
    ChangeSet, Component, DalContext, Func, FuncBinding, FuncId, HistoryActor, JwtSecretKey, Node,
    Prop, PropId, Schema, SchemaId, SchemaVariant, SchemaVariantId, StandardModel, User, UserClaim,
    UserPk, Visibility, Workspace, WorkspaceSignup,
};
use jwt_simple::algorithms::{RS256KeyPair, RSAKeyPairLike};
use jwt_simple::{claims::Claims, reexports::coarsetime::Duration};
use names::{Generator, Name};

pub mod builtins;
pub mod component_payload;

/// Commits the transactions in the given [`DalContext`] and returns a new context which reuses the
/// underlying [`dal::Connections`] and with identical state.
pub async fn commit_and_continue(ctx: DalContext) -> DalContext {
    ctx.commit_and_continue()
        .await
        .expect("unable to commit and continue")
}

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

// FIXME: this is horrifying, we need to do this properly, without a hardcoded key... etc
pub fn create_jwt(claim: UserClaim) -> String {
    let key = "-----BEGIN RSA PRIVATE KEY-----
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
-----END RSA PRIVATE KEY-----";
    let key_pair = RS256KeyPair::from_pem(key).expect("unable to extract private key");
    let claim = Claims::with_custom_claims(claim, Duration::from_days(1))
        .with_audience("https://app.systeminit.com")
        .with_issuer("https://app.systeminit.com")
        .with_subject(claim.user_pk);

    key_pair.sign(claim).expect("unable to sign jwt")
}

pub async fn workspace_signup(
    ctx: &DalContext,
    _jwt_secret_key: &JwtSecretKey,
) -> Result<(WorkspaceSignup, String)> {
    use color_eyre::eyre::WrapErr;

    let mut ctx = ctx.clone_with_head();

    let workspace_name = generate_fake_name();
    let user_name = format!("frank {workspace_name}");
    let user_email = format!("{workspace_name}@example.com");

    let nw = Workspace::signup(&mut ctx, &workspace_name, &user_name, &user_email)
        .await
        .wrap_err("cannot signup a new workspace")?;
    let auth_token = create_jwt(UserClaim {
        user_pk: nw.user.pk(),
        workspace_pk: *nw.workspace.pk(),
    });
    Ok((nw, auth_token))
}

pub async fn create_user(ctx: &DalContext) -> User {
    let name = generate_fake_name();
    User::new(
        ctx,
        UserPk::generate(),
        &name,
        &format!("{name}@test.systeminit.com"),
    )
    .await
    .expect("cannot create user")
}

pub async fn create_change_set(ctx: &DalContext) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(ctx, &name, None)
        .await
        .expect("cannot create change_set")
}

pub fn create_visibility_for_change_set(change_set: &ChangeSet) -> Visibility {
    Visibility::new(change_set.pk, None)
}

/// Creates a new [`Visibility`] backed by a new [`ChangeSet`]
pub async fn create_visibility_for_new_change_set(ctx: &DalContext) -> Visibility {
    let _history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(ctx).await;

    create_visibility_for_change_set(&change_set)
}

pub async fn create_change_set_and_update_ctx(ctx: &mut DalContext) {
    let visibility = create_visibility_for_new_change_set(ctx).await;
    ctx.update_visibility(visibility);
}

pub async fn create_component_and_node_for_schema(
    ctx: &DalContext,
    schema_id: &SchemaId,
) -> (Component, Node) {
    let name = generate_fake_name();
    let (component, node) = Component::new_for_default_variant_from_schema(ctx, &name, *schema_id)
        .await
        .expect("cannot create component");
    (component, node)
}

pub async fn create_component_for_schema(ctx: &DalContext, schema_id: &SchemaId) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_default_variant_from_schema(ctx, &name, *schema_id)
        .await
        .expect("cannot create component");
    component
}

pub async fn find_schema_by_name(ctx: &DalContext, schema_name: impl AsRef<str>) -> Schema {
    Schema::find_by_attr(ctx, "name", &schema_name.as_ref())
        .await
        .expect("cannot find schema by name")
        .pop()
        .expect("no schema found")
}

pub async fn find_schema_and_default_variant_by_name(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
) -> (Schema, SchemaVariant) {
    let schema = find_schema_by_name(ctx, schema_name).await;
    let default_variant = schema
        .default_variant(ctx)
        .await
        .expect("cannot get default schema variant");
    (schema, default_variant)
}

/// Get the "si:identity" [`Func`] and execute (if necessary).
pub async fn setup_identity_func(
    ctx: &DalContext,
) -> (
    FuncId,
    FuncBindingId,
    FuncBindingReturnValueId,
    FuncArgumentId,
) {
    let identity_func: Func = Func::find_by_attr(ctx, "name", &"si:identity".to_string())
        .await
        .expect("could not find identity func by name attr")
        .pop()
        .expect("identity func not found");

    let identity_func_identity_arg = FuncArgument::list_for_func(ctx, *identity_func.id())
        .await
        .expect("cannot list identity func args")
        .pop()
        .expect("cannot find identity func identity arg");

    let (identity_func_binding, identity_func_binding_return_value) =
        FuncBinding::create_and_execute(
            ctx,
            serde_json::json![{ "identity": null }],
            *identity_func.id(),
        )
        .await
        .expect("could not find or create identity func binding");
    (
        *identity_func.id(),
        *identity_func_binding.id(),
        *identity_func_binding_return_value.id(),
        *identity_func_identity_arg.id(),
    )
}

/// Find a [`PropId`] and its parent `PropId` by name. This only works if a parent [`Prop`] exists.
/// If a `Prop` and its parent share the same name and further precision is desired, you can
/// specify an optional "grandparent" `Prop` name.
///
/// _Use with caution!_
pub async fn find_prop_and_parent_by_name(
    ctx: &DalContext,
    prop_name: &str,
    parent_prop_name: &str,
    grandparent_prop_name: Option<&str>,
    schema_variant_id: SchemaVariantId,
) -> Option<(PropId, PropId)> {
    // Internal grandparent prop name check function.
    async fn check_grandparent(
        ctx: &DalContext,
        grandparent_prop_name: &str,
        parent_prop: &Prop,
    ) -> bool {
        if let Some(grandparent_prop) = parent_prop
            .parent_prop(ctx)
            .await
            .expect("could not find parent prop")
        {
            if grandparent_prop.name() == grandparent_prop_name {
                return true;
            }
        }
        false
    }

    // Begin to look through all props in the schema variant.
    for prop in SchemaVariant::get_by_id(ctx, &schema_variant_id)
        .await
        .expect("could not find schema variant")
        .expect("schema variant not found by id")
        .all_props(ctx)
        .await
        .expect("could not find all props for schema variant")
    {
        if let Some(parent_prop) = prop
            .parent_prop(ctx)
            .await
            .expect("could not find parent prop")
        {
            // Check if grandparent is valid. "Ignore" the check if not provided.
            let valid_grandparent_or_ignore = match grandparent_prop_name {
                Some(grandparent_prop_name) => {
                    check_grandparent(ctx, grandparent_prop_name, &parent_prop).await
                }
                None => true,
            };

            if prop.name() == prop_name
                && parent_prop.name() == parent_prop_name
                && valid_grandparent_or_ignore
            {
                return Some((*prop.id(), *parent_prop.id()));
            }
        }
    }
    None
}
