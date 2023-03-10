use color_eyre::Result;
use dal::{
    func::{
        argument::{FuncArgument, FuncArgumentId},
        binding::FuncBindingId,
        binding_return_value::FuncBindingReturnValueId,
    },
    ChangeSet, Component, DalContext, Func, FuncBinding, FuncId, HistoryActor, JwtSecretKey, Node,
    Prop, PropId, Schema, SchemaId, SchemaVariant, SchemaVariantId, StandardModel, User,
    Visibility, Workspace, WorkspaceSignup,
};
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

pub async fn workspace_signup(
    ctx: &DalContext,
    jwt_secret_key: &JwtSecretKey,
) -> Result<(WorkspaceSignup, String)> {
    use color_eyre::eyre::WrapErr;

    let mut ctx = ctx.clone_with_head();

    let workspace_name = generate_fake_name();
    let user_name = format!("frank {workspace_name}");
    let user_email = format!("{workspace_name}@example.com");
    let user_password = "snakes";

    let nw = Workspace::signup(
        &mut ctx,
        &workspace_name,
        &user_name,
        &user_email,
    )
    .await
    .wrap_err("cannot signup a new workspace")?;
    let auth_token = nw
        .user
        .login(&ctx, jwt_secret_key, "snakes")
        .await
        .wrap_err("cannot log in newly created user")?;
    Ok((nw, auth_token))
}

pub async fn create_user(ctx: &DalContext) -> User {
    let name = generate_fake_name();
    User::new(
        ctx,
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
