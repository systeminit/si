//! This module contains helpers for use when authoring dal integration tests.

use color_eyre::eyre::eyre;
use color_eyre::Result;
use dal::key_pair::KeyPairPk;
use dal::{
    AttributeValue, Component, ComponentId, DalContext, InputSocket, KeyPair, OutputSocket, Schema,
    SchemaVariant, SchemaVariantId, User, UserPk,
};
use names::{Generator, Name};

mod change_set;
mod property_editor_test_view;

pub use change_set::ChangeSetTestHelpers;
pub use property_editor_test_view::PropEditorTestView;

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
    ($str:expr) => {
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

/// Creates a [`Component`] from the default [`SchemaVariant`] corresponding to a provided
/// [`Schema`] name.
pub async fn create_component_for_schema_name(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Result<Component> {
    let schema = Schema::find_by_name(ctx, schema_name)
        .await?
        .ok_or(eyre!("schema not found"))?;
    let schema_variant_id = SchemaVariant::get_default_id_for_schema(ctx, schema.id()).await?;
    Ok(Component::new(ctx, name.as_ref().to_string(), schema_variant_id).await?)
}

/// Creates a [`Component`] for a given [`SchemaVariantId`](SchemaVariant).
pub async fn create_component_for_schema_variant(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> Result<Component> {
    let name = generate_fake_name()?;
    Ok(Component::new(ctx, &name, schema_variant_id).await?)
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

    Component::connect(
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
    let component = Component::get_by_id(ctx, component_id).await?;
    let component_input_sockets = component.input_socket_attribute_values(ctx).await?;
    let input_socket = InputSocket::find_with_name(ctx, input_socket_name, schema_variant_id)
        .await?
        .ok_or(eyre!("no input socket found"))?;
    let input_socket_match = component_input_sockets
        .get(&input_socket.id())
        .ok_or(eyre!("no input socket match found"))?;
    let input_socket_av =
        AttributeValue::get_by_id(ctx, input_socket_match.attribute_value_id).await?;
    Ok(input_socket_av.view(ctx).await?)
}

/// Gets the [`Value`] for a specific [`Component`]'s [`OutputSocket`] by the [`OutputSocket`] name
pub async fn get_component_output_socket_value(
    ctx: &DalContext,
    component_id: ComponentId,
    output_socket_name: impl AsRef<str>,
) -> Result<Option<serde_json::Value>> {
    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let component = Component::get_by_id(ctx, component_id).await?;
    let component_output_sockets = component.output_socket_attribute_values(ctx).await?;
    let output_socket = OutputSocket::find_with_name(ctx, output_socket_name, schema_variant_id)
        .await?
        .ok_or(eyre!("no output socket found"))?;
    let output_socket_match = component_output_sockets
        .get(&output_socket.id())
        .ok_or(eyre!("no output socket match found"))?;
    let output_socket_av =
        AttributeValue::get_by_id(ctx, output_socket_match.attribute_value_id).await?;
    Ok(output_socket_av.view(ctx).await?)
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

    let last_synced_value = AttributeValue::get_by_id(ctx, attribute_value_id)
        .await?
        .view(ctx)
        .await?;
    Ok(last_synced_value)
}
