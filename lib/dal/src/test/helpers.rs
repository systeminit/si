use names::{Generator, Name};
use serde_json::Value;
use std::collections::HashMap;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::binding::FuncBindingId;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::{
    node::NodeId, AttributeContext, AttributeReadContext, AttributeValue, AttributeValueId,
    BillingAccount, BillingAccountId, BillingAccountSignup, ChangeSet, Component, ComponentId,
    ComponentView, DalContext, DalContextBuilder, EditSession, Func, FuncBinding, FuncId, Group,
    HistoryActor, JwtSecretKey, Node, Prop, PropId, RequestContext, Schema, SchemaId,
    SchemaVariant, SchemaVariantId, StandardModel, System, Transactions, User, Visibility,
    WorkspaceId,
};

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn create_application(
    builder: &DalContextBuilder,
    txns: &Transactions<'_>,
    nba: &BillingAccountSignup,
) -> Node {
    let request_context = RequestContext::new_workspace_head(
        txns.pg(),
        HistoryActor::SystemInit,
        *nba.workspace.id(),
        None,
    )
    .await
    .expect("failed to create new workspace head request context");
    let ctx = builder.build(request_context, txns);

    let (_, node) = Component::new_application_with_node(&ctx, generate_fake_name())
        .await
        .expect("cannot create new application");
    node
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
        &ctx,
        &billing_account_name,
        &user_name,
        &user_email,
        &user_password,
    )
    .await
    .expect("cannot signup a new billing_account");
    let auth_token = nba
        .user
        .login(&ctx, jwt_secret_key, nba.billing_account.id(), "snakes")
        .await
        .expect("cannot log in newly created user");
    (nba, auth_token)
}

pub async fn create_group(ctx: &DalContext<'_, '_>) -> Group {
    let name = generate_fake_name();
    Group::new(ctx, &name).await.expect("cannot create group")
}

pub async fn create_user(ctx: &DalContext<'_, '_>) -> User {
    let name = generate_fake_name();
    User::new(
        ctx,
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
    BillingAccount::new(ctx, name, None)
        .await
        .expect("cannot create billing_account")
}

pub async fn create_billing_account(ctx: &DalContext<'_, '_>) -> BillingAccount {
    let name = generate_fake_name();
    create_billing_account_with_name(ctx, name).await
}

pub async fn create_change_set(
    ctx: &DalContext<'_, '_>,
    _billing_account_id: BillingAccountId,
) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(ctx, &name, None)
        .await
        .expect("cannot create change_set")
}

pub async fn create_edit_session(ctx: &DalContext<'_, '_>, change_set: &ChangeSet) -> EditSession {
    let name = generate_fake_name();
    EditSession::new(ctx, &change_set.pk, &name, None)
        .await
        .expect("cannot create edit_session")
}

pub async fn create_change_set_and_edit_session(
    ctx: &DalContext<'_, '_>,
    billing_account_id: BillingAccountId,
) -> (ChangeSet, EditSession) {
    let change_set = create_change_set(ctx, billing_account_id).await;
    let edit_session = create_edit_session(ctx, &change_set).await;
    (change_set, edit_session)
}

pub fn create_visibility_for_change_set_and_edit_session(
    change_set: &ChangeSet,
    edit_session: &EditSession,
) -> Visibility {
    Visibility::new(change_set.pk, edit_session.pk, None)
}

/// Creates a new [`Visibility`] backed by a new [`ChangeSet`] and a new [`EditSession`].
pub async fn create_visibility_for_new_change_set_and_edit_session(
    ctx: &DalContext<'_, '_>,
    billing_account_id: BillingAccountId,
) -> Visibility {
    let _history_actor = HistoryActor::SystemInit;
    let (change_set, edit_session) =
        create_change_set_and_edit_session(ctx, billing_account_id).await;

    create_visibility_for_change_set_and_edit_session(&change_set, &edit_session)
}

/// Creates a new [`DalContext`] in a change set and edit session in the given billing account.
pub async fn create_ctx_for_new_change_set_and_edit_session<'s, 't>(
    builder: &'s DalContextBuilder,
    txns: &'t Transactions<'t>,
    nba: &BillingAccountSignup,
    application_node_id: NodeId,
) -> DalContext<'s, 't> {
    let request_context = RequestContext::new_workspace_head(
        txns.pg(),
        HistoryActor::SystemInit,
        *nba.workspace.id(),
        Some(application_node_id),
    )
    .await
    .expect("failed to create request context");
    let ctx = builder.build(request_context, txns);
    let visibility =
        create_visibility_for_new_change_set_and_edit_session(&ctx, *nba.billing_account.id())
            .await;
    ctx.clone_with_new_visibility(visibility)
}

pub async fn new_ctx_for_new_change_set_and_edit_session<'a, 'b>(
    ctx: &DalContext<'a, 'b>,
    billing_account_id: BillingAccountId,
) -> DalContext<'a, 'b> {
    let visibility =
        create_visibility_for_new_change_set_and_edit_session(ctx, billing_account_id).await;
    ctx.clone_with_new_visibility(visibility)
}

pub async fn create_component_for_schema(
    ctx: &DalContext<'_, '_>,
    schema_id: &SchemaId,
) -> Component {
    let name = generate_fake_name();
    let (component, _, _) = Component::new_for_schema_with_node(ctx, &name, schema_id)
        .await
        .expect("cannot create component");
    component
        .set_schema(ctx, schema_id)
        .await
        .expect("cannot set the schema for our component");
    component
}

pub async fn create_system(ctx: &DalContext<'_, '_>) -> System {
    let name = generate_fake_name();
    System::new(ctx, name).await.expect("cannot create system")
}

pub async fn create_system_with_node(
    ctx: &DalContext<'_, '_>,
    wid: &WorkspaceId,
) -> (System, Node) {
    let name = generate_fake_name();
    System::new_with_node(ctx, name, wid)
        .await
        .expect("cannot create system")
}

pub async fn find_schema_by_name(ctx: &DalContext<'_, '_>, schema_name: impl AsRef<str>) -> Schema {
    Schema::find_by_attr(ctx, "name", &schema_name.as_ref())
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
        .default_variant(ctx)
        .await
        .expect("cannot get default schema variant");
    (schema, default_variant)
}

/// Update a [`AttributeValue`](crate::AttributeValue). This only works if the parent [`AttributeValue`]
/// for the same context corresponds to an _"object"_ [`Prop`](crate::Prop).
pub async fn update_attribute_value_for_prop_and_context(
    ctx: &DalContext<'_, '_>,
    prop_id: PropId,
    value: Option<Value>,
    base_attribute_read_context: AttributeReadContext,
) -> AttributeValueId {
    let attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(prop_id),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");

    let parent_prop = Prop::get_by_id(ctx, &prop_id)
        .await
        .expect("could not get prop by id")
        .expect("prop not found by id")
        .parent_prop(ctx)
        .await
        .expect("could not find parent prop")
        .expect("parent prop not found or prop does not have parent");
    let parent_attribute_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*parent_prop.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get attribute value")
    .expect("attribute value not found");

    let update_attribute_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(prop_id)
        .to_context()
        .expect("could not convert builder to attribute context");

    let (_, updated_attribute_value_id, task) = AttributeValue::update_for_context(
        ctx,
        *attribute_value.id(),
        Some(*parent_attribute_value.id()),
        update_attribute_context,
        value,
        None,
    )
    .await
    .expect("cannot update value for context");
    let _ = task
        .run_updates_in_ctx(ctx)
        .await
        .expect("unable to run dependent values async task");

    // Return the updated attribute value id.
    updated_attribute_value_id
}

/// Get the "si:identity" [`Func`](crate::Func) and execute (if necessary).
pub async fn setup_identity_func(
    ctx: &DalContext<'_, '_>,
) -> (FuncId, FuncBindingId, FuncBindingReturnValueId) {
    let identity_func: Func = Func::find_by_attr(ctx, "name", &"si:identity".to_string())
        .await
        .expect("could not find identity func by name attr")
        .pop()
        .expect("identity func not found");
    let (identity_func_binding, identity_func_binding_return_value) =
        FuncBinding::find_or_create_and_execute(
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
    )
}

/// Find a [`PropId`](crate::Prop) and its parent [`PropId`](crate::Prop) by name. This only works
/// if a parent [`Prop`](crate::Prop) exists. If a [`Prop`](crate::Prop) and its parent share the
/// same name and further precision is desired, you can specify an optional "grandparent"
/// [`Prop`](crate::Prop) name.
///
/// _Use with caution!_
pub async fn find_prop_and_parent_by_name(
    ctx: &DalContext<'_, '_>,
    prop_name: &str,
    parent_prop_name: &str,
    grandparent_prop_name: Option<&str>,
    schema_variant_id: SchemaVariantId,
) -> Option<(PropId, PropId)> {
    // Internal grandparent prop name check function.
    async fn check_grandparent(
        ctx: &DalContext<'_, '_>,
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

/// Payload used for bundling a [`Component`](crate::Component) with all metadata needed for a test.
pub struct ComponentPayload {
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub component_id: ComponentId,
    /// A map that uses [`Prop`](crate::Prop) "json pointer names" as keys and their ids as values.
    pub prop_map: HashMap<&'static str, PropId>,
    /// An [`AttributeReadContext`](crate::AttributeReadContext) that can be used for generating
    /// a [`ComponentView`](crate::ComponentView).
    pub base_attribute_read_context: AttributeReadContext,
}

impl ComponentPayload {
    /// Get the [`PropId`](crate::Prop) (value) corresponding to the "json pointer name" (key)
    /// in the "prop_map".
    pub fn get_prop_id(&self, prop_name: &str) -> PropId {
        *self
            .prop_map
            .get(prop_name)
            .expect("could not find PropId for key")
    }

    /// Merge the base [`AttributeReadContext`](crate::AttributeReadContext) with the
    /// [`PropId`](crate::Prop) found.
    pub fn attribute_read_context_with_prop_id(&self, prop_name: &str) -> AttributeReadContext {
        AttributeReadContext {
            prop_id: Some(self.get_prop_id(prop_name)),
            ..self.base_attribute_read_context
        }
    }

    /// Merge the base [`AttributeReadContext`](crate::AttributeReadContext) with the
    /// [`PropId`](crate::Prop) found and convert into an
    /// [`AttributeContext`](crate::AttributeContext).
    pub fn attribute_context_with_prop_id(&self, prop_name: &str) -> AttributeContext {
        AttributeContextBuilder::from(self.base_attribute_read_context)
            .set_prop_id(self.get_prop_id(prop_name))
            .to_context()
            .expect("could not convert builder to attribute context")
    }

    /// Generates a new [`ComponentView`](crate::ComponentView) and returns the "properites" field.
    pub async fn component_view_properties(&self, ctx: &DalContext<'_, '_>) -> serde_json::Value {
        ComponentView::for_context(ctx, self.base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties
    }
}
