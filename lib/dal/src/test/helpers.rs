use std::collections::HashMap;

use color_eyre::Result;
use names::{Generator, Name};
use serde_json::Value;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::argument::{FuncArgument, FuncArgumentId};
use crate::func::binding::FuncBindingId;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::node::NodeId;
use crate::{
    AttributeContext, AttributeReadContext, AttributeValue, AttributeValueId, BillingAccount,
    BillingAccountId, BillingAccountSignup, ChangeSet, Component, ComponentId, ComponentView,
    DalContext, DalContextBuilder, Func, FuncBinding, FuncId, Group, HistoryActor, JwtSecretKey,
    Node, Prop, PropId, RequestContext, Schema, SchemaId, SchemaVariant, SchemaVariantId,
    StandardModel, System, User, Visibility, WorkspaceId,
};

pub mod builtins;

/// Commits the transactions in the given [`DalContext`] and returns a new context which reuses the
/// underlying [`crate::Connections`] and with identical state.
pub async fn commit_and_continue(ctx: DalContext) -> DalContext {
    let (builder, conns, request_ctx) = ctx
        .commit_into_parts()
        .await
        .expect("failed to commit txns in dal context");

    builder
        .build_with_conns(request_ctx, conns)
        .await
        .expect("failed to build ctx")
}

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn billing_account_signup(
    ctx: &DalContext,
    jwt_secret_key: &JwtSecretKey,
) -> Result<(BillingAccountSignup, String)> {
    use color_eyre::eyre::WrapErr;

    let ctx = ctx.clone_with_universal_head();

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
    .wrap_err("cannot signup a new billing_account")?;
    let auth_token = nba
        .user
        .login(&ctx, jwt_secret_key, nba.billing_account.id(), "snakes")
        .await
        .wrap_err("cannot log in newly created user")?;
    Ok((nba, auth_token))
}

pub async fn create_group(ctx: &DalContext) -> Group {
    let name = generate_fake_name();
    Group::new(ctx, &name).await.expect("cannot create group")
}

pub async fn create_user(ctx: &DalContext) -> User {
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
    ctx: &DalContext,
    name: impl AsRef<str>,
) -> BillingAccount {
    BillingAccount::new(ctx, name, None)
        .await
        .expect("cannot create billing_account")
}

pub async fn create_billing_account(ctx: &DalContext) -> BillingAccount {
    let name = generate_fake_name();
    create_billing_account_with_name(ctx, name).await
}

pub async fn create_change_set(
    ctx: &DalContext,
    _billing_account_id: BillingAccountId,
) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(ctx, &name, None)
        .await
        .expect("cannot create change_set")
}

pub fn create_visibility_for_change_set(change_set: &ChangeSet) -> Visibility {
    Visibility::new(change_set.pk, None)
}

/// Creates a new [`Visibility`] backed by a new [`ChangeSet`]
pub async fn create_visibility_for_new_change_set(
    ctx: &DalContext,
    billing_account_id: BillingAccountId,
) -> Visibility {
    let _history_actor = HistoryActor::SystemInit;
    let change_set = create_change_set(ctx, billing_account_id).await;

    create_visibility_for_change_set(&change_set)
}

pub async fn create_change_set_and_update_ctx(ctx: &mut DalContext, nba: &BillingAccountSignup) {
    ctx.update_to_workspace_tenancies(*nba.workspace.id())
        .await
        .expect("failed to update dal context to workspace tenancies");
    let visibility = create_visibility_for_new_change_set(ctx, *nba.billing_account.id()).await;
    ctx.update_visibility(visibility);
}

/// Creates a new [`DalContext`] in a change set and edit session in the given billing account.
pub async fn create_ctx_for_new_change_set(
    builder: &DalContextBuilder,
    nba: &BillingAccountSignup,
) -> DalContext {
    let mut ctx = builder
        .build(RequestContext::new_universal_head(HistoryActor::SystemInit))
        .await
        .expect("failed to build dal context");
    ctx.update_to_workspace_tenancies(*nba.workspace.id())
        .await
        .expect("failed to update dal context to workspace tenancies");

    let visibility = create_visibility_for_new_change_set(&ctx, *nba.billing_account.id()).await;
    ctx.update_visibility(visibility);
    ctx
}

pub async fn new_ctx_for_new_change_set(
    ctx: &DalContext,
    billing_account_id: BillingAccountId,
) -> DalContext {
    let visibility = create_visibility_for_new_change_set(ctx, billing_account_id).await;
    ctx.clone_with_new_visibility(visibility)
}

pub async fn create_component_and_node_for_schema(
    ctx: &DalContext,
    schema_id: &SchemaId,
) -> (Component, Node) {
    let name = generate_fake_name();
    let (component, node) = Component::new_for_schema_with_node(ctx, &name, schema_id)
        .await
        .expect("cannot create component");
    (component, node)
}

pub async fn create_component_for_schema(ctx: &DalContext, schema_id: &SchemaId) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_schema_with_node(ctx, &name, schema_id)
        .await
        .expect("cannot create component");
    component
}

pub async fn create_system(ctx: &DalContext) -> System {
    let name = generate_fake_name();
    System::new(ctx, name).await.expect("cannot create system")
}

pub async fn create_system_with_node(ctx: &DalContext, wid: &WorkspaceId) -> (System, Node) {
    let name = generate_fake_name();
    System::new_with_node(ctx, name, wid)
        .await
        .expect("cannot create system")
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

/// Get the "si:identity" [`Func`](crate::Func) and execute (if necessary).
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

    let (identity_func_binding, identity_func_binding_return_value, _) =
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
        *identity_func_identity_arg.id(),
    )
}

/// Find a [`PropId`](crate::Prop) and its parent [`PropId`](crate::Prop) by name. This only works
/// if a parent [`Prop`](crate::Prop) exists. If a [`Prop`](crate::Prop) and its parent share the
/// same name and further precision is desired, you can specify an optional "grandparent"
/// [`Prop`](crate::Prop) name.
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

/// Payload used for bundling a [`Component`](crate::Component) with all metadata needed for a test.
pub struct ComponentPayload {
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub component_id: ComponentId,
    pub node_id: NodeId,
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
    pub async fn component_view_properties(&self, ctx: &DalContext) -> serde_json::Value {
        ComponentView::for_context(ctx, self.base_attribute_read_context)
            .await
            .expect("cannot get component view")
            .properties
    }

    /// Update a [`AttributeValue`](crate::AttributeValue). This only works if the parent
    /// [`AttributeValue`] for the same context corresponds to an _"object"_ [`Prop`](crate::Prop).
    pub async fn update_attribute_value_for_prop_name(
        &self,
        ctx: &DalContext,
        prop_name: impl AsRef<str>,
        value: Option<Value>,
    ) -> AttributeValueId {
        let prop_id = self.get_prop_id(prop_name.as_ref());

        let attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(prop_id),
                ..self.base_attribute_read_context
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
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let update_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(prop_id)
                .to_context()
                .expect("could not convert builder to attribute context");

        let (_, updated_attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(*parent_attribute_value.id()),
            update_attribute_context,
            value,
            None,
        )
        .await
        .expect("cannot update value for context");

        // Return the updated attribute value id.
        updated_attribute_value_id
    }

    /// Inserts an [`AttributeValue`](crate::AttributeValue) corresponding to a _primitive_
    /// [`Prop`](crate::Prop) (string, boolean or integer) in an _array_ [`Prop`](crate::Prop).
    pub async fn insert_array_primitive_element(
        &self,
        ctx: &DalContext,
        array_prop_name: impl AsRef<str>,
        element_prop_name: impl AsRef<str>,
        value: Value,
    ) -> AttributeValueId {
        let array_prop_id = self.get_prop_id(array_prop_name.as_ref());
        let element_prop_id = self.get_prop_id(element_prop_name.as_ref());

        let array_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(array_prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let insert_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(element_prop_id)
                .to_context()
                .expect("could not create insert context");

        // Return the element attribute value id.
        AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *array_attribute_value.id(),
            Some(value),
            None,
        )
        .await
        .expect("could not insert object into array")
    }

    /// Inserts an [`AttributeValue`](crate::AttributeValue) corresponding to an "empty" _object_
    /// [`Prop`](crate::Prop) in an _array_ [`Prop`](crate::Prop).
    pub async fn insert_array_object_element(
        &self,
        ctx: &DalContext,
        array_prop_name: impl AsRef<str>,
        element_prop_name: impl AsRef<str>,
    ) -> AttributeValueId {
        let array_prop_id = self.get_prop_id(array_prop_name.as_ref());
        let element_prop_id = self.get_prop_id(element_prop_name.as_ref());

        let array_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext {
                prop_id: Some(array_prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let insert_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(element_prop_id)
                .to_context()
                .expect("could not create insert context");

        // Return the element attribute value id.
        AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *array_attribute_value.id(),
            Some(serde_json::json![{}]),
            None,
        )
        .await
        .expect("could not insert object into array")
    }

    /// Using the element [`AttributeValueId`](AttributeValueId) from
    /// [`Self::insert_array_object_element()`], update an [`AttributeValue`](crate::AttributeValue)
    /// corresponding to a "field" within the _object_ element.
    pub async fn update_attribute_value_for_prop_name_and_parent_element_attribute_value_id(
        &self,
        ctx: &DalContext,
        prop_name: impl AsRef<str>,
        value: Option<Value>,
        element_attribute_value_id: AttributeValueId,
    ) -> AttributeValueId {
        let prop_id = self.get_prop_id(prop_name.as_ref());
        let attribute_value = AttributeValue::find_with_parent_and_key_for_context(
            ctx,
            Some(element_attribute_value_id),
            None,
            AttributeReadContext {
                prop_id: Some(prop_id),
                ..self.base_attribute_read_context
            },
        )
        .await
        .expect("cannot get attribute value")
        .expect("attribute value not found");

        let update_attribute_context =
            AttributeContextBuilder::from(self.base_attribute_read_context)
                .set_prop_id(prop_id)
                .to_context()
                .expect("could not convert builder to attribute context");

        let (_, updated_attribute_value_id) = AttributeValue::update_for_context(
            ctx,
            *attribute_value.id(),
            Some(element_attribute_value_id),
            update_attribute_context,
            value,
            None,
        )
        .await
        .expect("cannot update value for context");

        // Return the updated attribute value id.
        updated_attribute_value_id
    }
}
