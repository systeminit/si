//! This module contains helpers for use when authoring dal integration tests.

use std::time::Duration;

use audit_database::{
    AuditDatabaseContext,
    AuditLogRow,
};
use color_eyre::{
    Result,
    eyre::eyre,
};
use dal::{
    AttributeValue,
    Component,
    ComponentId,
    ComponentType,
    DalContext,
    InputSocket,
    KeyPair,
    OutputSocket,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    UserPk,
    audit_logging,
    component::socket::{
        ComponentInputSocket,
        ComponentOutputSocket,
    },
    diagram::view::View,
    key_pair::KeyPairPk,
    schema::variant::authoring::VariantAuthoringClient,
};
use itertools::Itertools;
use names::{
    Generator,
    Name,
};
use si_data_nats::async_nats::jetstream::stream::Stream;
use si_db::User;
use tokio::time::Instant;

mod property_editor_test_view;

/// Test helpers for attribute values and prototypes
pub mod attribute;
/// Test helpers for change sets
pub mod change_set;
/// Test helpers for components
pub mod component;
/// Test helpers for funcs
pub mod func;
/// Test helpers for schemas
pub mod schema;
/// Test helpers for secrets
pub mod secret;

pub use change_set::ChangeSetTestHelpers;
use dal::diagram::view::ViewId;
pub use property_editor_test_view::PropEditorTestView;
use serde_json::Value;

/// Generates a fake name.
pub fn generate_fake_name() -> Result<String> {
    Generator::with_naming(Name::Numbered)
        .next()
        .ok_or(eyre!("could not generate fake name"))
}

/// Creates a connection annotation string.
#[allow(clippy::expect_used)]
#[macro_export]
macro_rules! connection_annotation_string {
    ($str:expr_2021) => {
        serde_json::to_string(&vec![$str]).expect("unable to parse annotation string")
    };
}

/// Creates a dummy key pair.
pub async fn create_key_pair(ctx: &DalContext) -> Result<KeyPair> {
    let name = generate_fake_name()?;
    Ok(KeyPair::new(ctx, &name).await?)
}

/// Creates a dummy user.
pub async fn create_user(ctx: &DalContext) -> Result<User> {
    let name = generate_fake_name()?;
    Ok(User::new(
        ctx,
        UserPk::generate(),
        &name,
        &format!("{name}@test.systeminit.com"),
        None::<&str>,
    )
    .await?)
}

/// Creates a dummy schema.
pub async fn create_schema(ctx: &DalContext) -> Result<Schema> {
    let name = generate_fake_name()?;
    Ok(Schema::new(ctx, &name).await?)
}

/// Finds the [`Schema`] with the given name, finds the default [`SchemaVariant`], creates an unlocked copy of it, and returns the [`SchemaVariantId`]
pub async fn create_unlocked_variant_copy_for_schema_name(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
) -> Result<SchemaVariantId> {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, schema_name).await?;
    let unlocked_copy_sv =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await?
            .id();
    Ok(unlocked_copy_sv)
}

/// Creates a [`Component`] from the default [`SchemaVariant`] corresponding to a provided
/// [`Schema`] name, in the default view
pub async fn create_component_for_default_schema_name_in_default_view(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Component> {
    let view_id = View::get_id_for_default(ctx).await?;
    create_component_for_default_schema_name(ctx, schema_name, name, view_id).await
}

/// Creates a [`Component`] from the default [`SchemaVariant`] corresponding to a provided
/// [`Schema`] name in the provided [dal::diagram::view::View]
pub async fn create_component_for_default_schema_name(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
    view_id: ViewId,
) -> Result<Component> {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, schema_name).await?;

    Ok(Component::new(ctx, name.as_ref().to_string(), schema_variant_id, view_id).await?)
}

/// Creates a [`Component`] from the default [`SchemaVariant`] corresponding to a provided
/// [`Schema`] name.
pub async fn create_component_for_unlocked_schema_name_on_default_view(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Component> {
    let schema = Schema::get_by_name(ctx, schema_name).await?;
    let schema_variant_id = SchemaVariant::get_unlocked_for_schema(ctx, schema.id())
        .await?
        .ok_or(eyre!("no unlocked schema variant for schema name"))?;
    let view_id = View::get_id_for_default(ctx).await?;

    Ok(Component::new(
        ctx,
        name.as_ref().to_string(),
        schema_variant_id.id(),
        view_id,
    )
    .await?)
}

/// Creates a [`Component`] from the default [`SchemaVariant`] corresponding to a provided
/// [`Schema`] name.
pub async fn create_component_for_schema_name_with_type_on_default_view(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
    component_type: ComponentType,
) -> Result<Component> {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, schema_name).await?;

    let view_id = View::get_id_for_default(ctx).await?;

    let component =
        Component::new(ctx, name.as_ref().to_string(), schema_variant_id, view_id).await?;
    Component::set_type_by_id_unchecked(ctx, component.id(), component_type).await?;
    Ok(component)
}

/// Creates a [`Component`] for a given [`SchemaVariantId`](SchemaVariant).
pub async fn create_component_for_schema_variant_on_default_view(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> Result<Component> {
    let name = generate_fake_name()?;

    create_named_component_for_schema_variant_on_default_view(ctx, name, schema_variant_id).await
}

/// Creates a [`Component`] for a given [`SchemaVariantId`](SchemaVariant).
pub async fn create_named_component_for_schema_variant_on_default_view(
    ctx: &DalContext,
    name: impl AsRef<str>,
    schema_variant_id: SchemaVariantId,
) -> Result<Component> {
    let view_id = View::get_id_for_default(ctx).await?;

    Ok(Component::new(ctx, name.as_ref().to_string(), schema_variant_id, view_id).await?)
}

/// Connects two [`Components`](Component) for a given set of socket names.
pub async fn connect_components_with_socket_names(
    ctx: &DalContext,
    source_component_id: ComponentId,
    output_socket_name: impl AsRef<str>,
    destination_component_id: ComponentId,
    input_socket_name: impl AsRef<str>,
) -> Result<()> {
    let from_socket_id = {
        let sv_id = Component::schema_variant_id(ctx, source_component_id).await?;
        OutputSocket::find_with_name(ctx, output_socket_name, sv_id)
            .await?
            .ok_or(eyre!("no output socket found"))?
            .id()
    };

    let to_socket_id = {
        let sv_id = Component::schema_variant_id(ctx, destination_component_id).await?;
        InputSocket::find_with_name(ctx, input_socket_name, sv_id)
            .await?
            .ok_or(eyre!("no input socket found"))?
            .id()
    };

    Component::connect_for_tests(
        ctx,
        source_component_id,
        from_socket_id,
        destination_component_id,
        to_socket_id,
    )
    .await?;
    Ok(())
}

/// Disconnects two [`Components`](Component) for a given set of socket names.
pub async fn disconnect_components_with_socket_names(
    ctx: &DalContext,
    source_component_id: ComponentId,
    output_socket_name: impl AsRef<str>,
    destination_component_id: ComponentId,
    input_socket_name: impl AsRef<str>,
) -> Result<()> {
    let from_socket_id = {
        let sv_id = Component::schema_variant_id(ctx, source_component_id).await?;
        OutputSocket::find_with_name(ctx, output_socket_name, sv_id)
            .await?
            .ok_or(eyre!("no output socket found"))?
            .id()
    };

    let to_socket_id = {
        let sv_id = Component::schema_variant_id(ctx, destination_component_id).await?;
        InputSocket::find_with_name(ctx, input_socket_name, sv_id)
            .await?
            .ok_or(eyre!("no input socket found"))?
            .id()
    };

    Component::remove_connection(
        ctx,
        source_component_id,
        from_socket_id,
        destination_component_id,
        to_socket_id,
    )
    .await?;
    Ok(())
}

/// Gets the [`Value`] for a specific [`Component`]'s [`InputSocket`] by the [`InputSocket`] name
pub async fn get_component_input_socket_value(
    ctx: &DalContext,
    component_id: ComponentId,
    input_socket_name: impl AsRef<str>,
) -> Result<Option<serde_json::Value>> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let component_input_sockets =
        ComponentInputSocket::list_for_component_id(ctx, component_id).await?;
    let input_socket = InputSocket::find_with_name(ctx, input_socket_name, schema_variant_id)
        .await?
        .ok_or(eyre!("no input socket found"))?;
    let component_input_socket = component_input_sockets
        .into_iter()
        .filter(|socket| socket.input_socket_id == input_socket.id())
        .collect_vec()
        .pop()
        .ok_or(eyre!("no input socket match found"))?;
    AttributeValue::view(ctx, component_input_socket.attribute_value_id)
        .await
        .map_err(Into::into)
}
/// Gets the [`Value`] for a specific [`Component`]'s [`InputSocket`] by the [`InputSocket`] name
pub async fn get_component_input_socket_attribute_value(
    ctx: &DalContext,
    component_id: ComponentId,
    input_socket_name: impl AsRef<str>,
) -> Result<AttributeValue> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let component_input_sockets =
        ComponentInputSocket::list_for_component_id(ctx, component_id).await?;
    let input_socket = InputSocket::find_with_name(ctx, input_socket_name, schema_variant_id)
        .await?
        .ok_or(eyre!("no input socket found"))?;
    let component_input_socket = component_input_sockets
        .into_iter()
        .filter(|socket| socket.input_socket_id == input_socket.id())
        .collect_vec()
        .pop()
        .ok_or(eyre!("no input socket match found"))?;
    let input_socket_av =
        AttributeValue::get_by_id(ctx, component_input_socket.attribute_value_id).await?;
    Ok(input_socket_av)
}

/// Gets the [`Value`] for a specific [`Component`]'s [`OutputSocket`] by the [`OutputSocket`] name
pub async fn get_component_output_socket_value(
    ctx: &DalContext,
    component_id: ComponentId,
    output_socket_name: impl AsRef<str>,
) -> Result<Option<serde_json::Value>> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let component_output_sockets =
        ComponentOutputSocket::list_for_component_id(ctx, component_id).await?;
    let output_socket = OutputSocket::find_with_name(ctx, output_socket_name, schema_variant_id)
        .await?
        .ok_or(eyre!("no output socket found"))?;
    let component_output_socket = component_output_sockets
        .into_iter()
        .filter(|socket| socket.output_socket_id == output_socket.id())
        .collect_vec()
        .pop()
        .ok_or(eyre!("no input socket match found"))?;
    AttributeValue::view(ctx, component_output_socket.attribute_value_id)
        .await
        .map_err(Into::into)
}

/// Update the [`Value`] for a specific [`AttributeValue`] for the given [`Component`](ComponentId) by the [`PropPath`]
pub async fn update_attribute_value_for_component(
    ctx: &DalContext,
    component_id: ComponentId,
    prop_path: &[&str],
    value: serde_json::Value,
) -> Result<()> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let mut attribute_value_ids = component.attribute_values_for_prop(ctx, prop_path).await?;
    let attribute_value_id = attribute_value_ids
        .pop()
        .ok_or(eyre!("unexpected: no attribute values found"))?;
    if !attribute_value_ids.is_empty() {
        return Err(eyre!("unexpected: more than one attribute value found"));
    }
    AttributeValue::update(ctx, attribute_value_id, Some(value)).await?;
    Ok(())
}

/// Given a [`ComponentId`] and PropPath, get the value for an attribute value at that path
pub async fn get_attribute_value_for_component(
    ctx: &DalContext,
    component_id: ComponentId,
    prop_path: &[&str],
) -> Result<Value> {
    get_attribute_value_for_component_opt(ctx, component_id, prop_path)
        .await?
        .ok_or(eyre!("unexpected: missing attribute value"))
}

/// Given a [`ComponentId`] and PropPath, get the value for an attribute value at that path
pub async fn get_attribute_value_for_component_opt(
    ctx: &DalContext,
    component_id: ComponentId,
    prop_path: &[&str],
) -> Result<Option<Value>> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let mut attribute_value_ids = component.attribute_values_for_prop(ctx, prop_path).await?;
    let attribute_value_id = attribute_value_ids
        .pop()
        .ok_or(eyre!("unexpected, no attribute values found for prop"))?;
    assert!(attribute_value_ids.is_empty());

    AttributeValue::view(ctx, attribute_value_id)
        .await
        .map_err(Into::into)
}

/// Encrypts a message with a given [`KeyPairPk`](KeyPair).
pub async fn encrypt_message(
    ctx: &DalContext,
    key_pair_pk: KeyPairPk,
    message: &serde_json::Value,
) -> Result<Vec<u8>> {
    let public_key = KeyPair::get_by_pk(ctx, key_pair_pk).await?;

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(message)?,
        public_key.public_key(),
    );
    Ok(crypted)
}

/// Fetches the value stored at "/root/resource/last_synced" for the provided [`Component`].
pub async fn fetch_resource_last_synced_value(
    ctx: &DalContext,
    component_id: ComponentId,
) -> Result<Option<serde_json::Value>> {
    let mut attribute_value_ids = Component::attribute_values_for_prop_by_id(
        ctx,
        component_id,
        &["root", "resource", "last_synced"],
    )
    .await?;
    let attribute_value_id = attribute_value_ids
        .pop()
        .ok_or(eyre!("unexpected: no attribute values found"))?;
    if !attribute_value_ids.is_empty() {
        return Err(eyre!("unexpected: more than one attribute value found"));
    }

    AttributeValue::view(ctx, attribute_value_id)
        .await
        .map_err(Into::into)
}

/// Extracts the value and validation from a raw property edtior value.
pub fn extract_value_and_validation(
    prop_editor_value: serde_json::Value,
) -> Result<serde_json::Value> {
    let value = prop_editor_value
        .get("value")
        .ok_or(eyre!("get value from property editor value"))?;
    let validation = prop_editor_value
        .get("validation")
        .ok_or(eyre!("get validation from property editor value"))?;

    Ok(serde_json::json!({
        "value": value,
        "validation": validation,
    }))
}

/// Retries until no more messages are seen on the NATS JetStream stream.
pub async fn confirm_jetstream_stream_has_no_messages(
    stream: &Stream,
    timeout_seconds: u64,
    interval_milliseconds: u64,
) -> Result<()> {
    let timeout = Duration::from_secs(timeout_seconds);
    let interval = Duration::from_millis(interval_milliseconds);

    let start = Instant::now();
    let mut message_count = 0;

    while start.elapsed() < timeout {
        message_count = stream.get_info().await?.state.messages;
        if message_count == 0 {
            return Ok(());
        }
        tokio::time::sleep(interval).await;
    }

    Err(eyre!(
        "hit timeout and stream still has at least one message: {message_count}"
    ))
}

/// Retries listing audit logs until the expected number of rows are returned.
pub async fn list_audit_logs_until_expected_number_of_rows(
    ctx: &DalContext,
    context: &AuditDatabaseContext,
    size: usize,
    expected_number_of_rows: usize,
    timeout_seconds: u64,
    interval_milliseconds: u64,
) -> Result<Vec<AuditLogRow>> {
    let timeout = Duration::from_secs(timeout_seconds);
    let interval = Duration::from_millis(interval_milliseconds);

    let start = Instant::now();
    let mut actual_number_of_rows = 0;

    while start.elapsed() < timeout {
        let (audit_logs, _) = audit_logging::list(ctx, context, size, false).await?;
        actual_number_of_rows = audit_logs.len();
        if actual_number_of_rows == expected_number_of_rows {
            return Ok(audit_logs);
        }
        tokio::time::sleep(interval).await;
    }

    Err(eyre!(
        "hit timeout before audit logs query returns expected number of rows (expected: {expected_number_of_rows}, actual: {actual_number_of_rows})"
    ))
}
