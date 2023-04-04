use dal::{
    component::ComponentKind,
    func::{binding::FuncBinding, FuncId},
    jwt_key::JwtSecretKey,
    key_pair::KeyPairPk,
    node::NodeKind,
    schema,
    socket::{Socket, SocketArity, SocketEdgeKind, SocketKind},
    ChangeSet, ChangeSetPk, Component, DalContext, DiagramKind, EncryptedSecret, Func,
    FuncBackendKind, FuncBackendResponseType, KeyPair, Node, Prop, PropId, PropKind, Schema,
    SchemaId, SchemaVariantId, Secret, SecretKind, SecretObjectType, StandardModel, User,
    UserClaim, UserPk, Visibility, Workspace, WorkspacePk, WorkspaceSignup,
};
use jwt_simple::{
    algorithms::{RS256KeyPair, RSAKeyPairLike},
    claims::Claims,
    reexports::coarsetime::Duration,
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
    ctx: &mut DalContext,
    _jwt_secret_key: &JwtSecretKey,
) -> (WorkspaceSignup, String) {
    let workspace_name = generate_fake_name();
    let user_name = format!("frank {workspace_name}");
    let user_email = format!("{workspace_name}@example.com");

    let nw = Workspace::signup(ctx, &workspace_name, &user_name, &user_email)
        .await
        .expect("cannot signup a new workspace");
    let auth_token = create_jwt(UserClaim {
        user_pk: nw.user.pk(),
        workspace_pk: *nw.workspace.pk(),
    });
    (nw, auth_token)
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

/// Create a [`Prop`](dal::Prop) with a given [`PropKind`](dal::PropKind), name and parent
/// [`PropId`](dal::Prop).
pub async fn create_prop_and_set_parent(
    ctx: &DalContext,
    prop_kind: PropKind,
    name: impl AsRef<str>,
    parent_prop_id: PropId,
) -> Prop {
    let name = name.as_ref();
    let new_prop = Prop::new(ctx, name, prop_kind, None)
        .await
        .expect("cannot create prop");
    new_prop
        .set_parent_prop(ctx, parent_prop_id)
        .await
        .expect("cannot set parent to new prop");
    new_prop
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
