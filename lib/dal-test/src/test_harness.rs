use dal::{
    component::ComponentKind,
    func::{binding::FuncBinding, FuncId},
    key_pair::KeyPairPk,
    node::NodeKind,
    schema,
    socket::{Socket, SocketArity, SocketEdgeKind, SocketKind},
    ChangeSet, ChangeSetPk, Component, DalContext, DiagramKind, EncryptedSecret, Func,
    FuncBackendKind, FuncBackendResponseType, KeyPair, Node, Schema, SchemaId, SchemaVariantId,
    Secret, SecretKind, SecretObjectType, StandardModel, User, UserPk, Visibility, Workspace,
    WorkspacePk,
};
use names::{Generator, Name};

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

pub async fn create_change_set(ctx: &DalContext) -> ChangeSet {
    let name = generate_fake_name();
    ChangeSet::new(ctx, &name, None)
        .await
        .expect("cannot create change_set")
}

pub fn create_visibility_change_set(change_set: &ChangeSet) -> Visibility {
    Visibility::new(change_set.pk, None)
}

pub fn create_visibility_head() -> Visibility {
    Visibility::new(ChangeSetPk::NONE, None)
}

pub async fn create_workspace(ctx: &mut DalContext) -> Workspace {
    let name = generate_fake_name();
    Workspace::new(ctx, WorkspacePk::generate(), &name)
        .await
        .expect("cannot create workspace")
}

pub async fn create_key_pair(ctx: &DalContext) -> KeyPair {
    let name = generate_fake_name();
    KeyPair::new(ctx, &name)
        .await
        .expect("cannot create key_pair")
}

pub async fn create_user(ctx: &DalContext) -> User {
    let name = generate_fake_name();
    User::new(
        ctx,
        UserPk::generate(),
        &name,
        &format!("{name}@test.systeminit.com"),
        None::<&str>,
    )
    .await
    .expect("cannot create user")
}

pub async fn create_schema(ctx: &DalContext) -> Schema {
    let name = generate_fake_name();
    Schema::new(ctx, &name, &ComponentKind::Standard)
        .await
        .expect("cannot create schema")
}

pub async fn create_schema_variant(ctx: &DalContext, schema_id: SchemaId) -> schema::SchemaVariant {
    create_schema_variant_with_root(ctx, schema_id).await.0
}

pub async fn create_schema_variant_with_root(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> (schema::SchemaVariant, schema::RootProp) {
    let name = generate_fake_name();
    let (variant, root) = schema::SchemaVariant::new(ctx, schema_id, name)
        .await
        .expect("cannot create schema variant");

    let _input_socket = Socket::new(
        ctx,
        "input",
        SocketKind::Standalone,
        &SocketEdgeKind::ConfigurationInput,
        &SocketArity::Many,
        &DiagramKind::Configuration,
        Some(*variant.id()),
    )
    .await
    .expect("Unable to create socket");

    let _output_socket = Socket::new(
        ctx,
        "output",
        SocketKind::Standalone,
        &SocketEdgeKind::ConfigurationOutput,
        &SocketArity::Many,
        &DiagramKind::Configuration,
        Some(*variant.id()),
    )
    .await
    .expect("Unable to create socket");

    (variant, root)
}

pub async fn create_component_and_schema(ctx: &DalContext) -> Component {
    let schema = create_schema(ctx).await;
    let mut schema_variant = create_schema_variant(ctx, *schema.id()).await;
    schema_variant
        .finalize(ctx, None)
        .await
        .expect("unable to finalize schema variant");
    let name = generate_fake_name();
    let (component, _) = Component::new(ctx, &name, *schema_variant.id())
        .await
        .expect("cannot create component");
    component
}

pub async fn create_component_for_schema_variant(
    ctx: &DalContext,
    schema_variant_id: &SchemaVariantId,
) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new(ctx, &name, *schema_variant_id)
        .await
        .expect("cannot create component");
    component
}

pub async fn create_component_for_schema(ctx: &DalContext, schema_id: &SchemaId) -> Component {
    let name = generate_fake_name();
    let (component, _) = Component::new_for_default_variant_from_schema(ctx, &name, *schema_id)
        .await
        .expect("cannot create component");
    component
}

pub async fn create_node(ctx: &DalContext, node_kind: &NodeKind) -> Node {
    Node::new(ctx, node_kind).await.expect("cannot create node")
}

pub async fn create_func(ctx: &DalContext) -> Func {
    let name = generate_fake_name();
    Func::new(
        ctx,
        name,
        FuncBackendKind::String,
        FuncBackendResponseType::String,
    )
    .await
    .expect("cannot create func")
}

pub async fn create_func_binding(
    ctx: &DalContext,
    args: serde_json::Value,
    func_id: FuncId,
    backend_kind: FuncBackendKind,
) -> FuncBinding {
    FuncBinding::new(ctx, args, func_id, backend_kind)
        .await
        .expect("cannot create func")
}

pub async fn encrypt_message(
    ctx: &DalContext,
    key_pair_pk: KeyPairPk,
    message: &serde_json::Value,
) -> Vec<u8> {
    let public_key = KeyPair::get_by_pk(ctx, key_pair_pk)
        .await
        .expect("failed to fetch key pair");

    let crypted = sodiumoxide::crypto::sealedbox::seal(
        &serde_json::to_vec(message).expect("failed to serialize message"),
        public_key.public_key(),
    );
    crypted
}

pub async fn create_secret(ctx: &DalContext, key_pair_pk: KeyPairPk) -> Secret {
    let name = generate_fake_name();
    EncryptedSecret::new(
        ctx,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &encrypt_message(ctx, key_pair_pk, &serde_json::json!({ "name": name })).await,
        key_pair_pk,
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret")
}

pub async fn create_secret_with_message(
    ctx: &DalContext,
    key_pair_pk: KeyPairPk,
    message: &serde_json::Value,
) -> Secret {
    let name = generate_fake_name();
    EncryptedSecret::new(
        ctx,
        &name,
        SecretObjectType::Credential,
        SecretKind::DockerHub,
        &encrypt_message(ctx, key_pair_pk, message).await,
        key_pair_pk,
        Default::default(),
        Default::default(),
    )
    .await
    .expect("cannot create secret")
}
