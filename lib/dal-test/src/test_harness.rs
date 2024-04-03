use dal::property_editor::schema::{
    PropertyEditorProp, PropertyEditorPropKind, PropertyEditorSchema,
};
use dal::property_editor::values::{PropertyEditorValue, PropertyEditorValues};
use dal::property_editor::{PropertyEditorPropId, PropertyEditorValueId};
use dal::{
    func::{binding::FuncBinding, FuncId},
    key_pair::KeyPairPk,
    Component, ComponentId, DalContext, FuncBackendKind, InputSocket, KeyPair, OutputSocket,
    Schema, SchemaVariant, SchemaVariantId, User, UserPk,
};
use itertools::enumerate;
use jwt_simple::prelude::{Deserialize, Serialize};
use names::{Generator, Name};
use serde_json::Value;
use std::collections::HashMap;

pub async fn commit_and_update_snapshot(ctx: &mut DalContext) {
    let conflicts = ctx.blocking_commit().await.expect("unable to commit");
    assert!(conflicts.is_none());
    ctx.update_snapshot_to_visibility()
        .await
        .expect("unable to update snapshot to visibility");
}

pub fn generate_fake_name() -> String {
    Generator::with_naming(Name::Numbered).next().unwrap()
}

#[macro_export]
macro_rules! connection_annotation_string {
    ($str:expr) => {
        serde_json::to_string(&vec![$str]).expect("Unable to parse annotation string")
    };
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
    Schema::new(ctx, &name).await.expect("cannot create schema")
}

// pub async fn create_schema_variant(ctx: &DalContext, schema_id: SchemaId) -> SchemaVariant {
//     create_schema_variant_with_root(ctx, schema_id).await.0
// }

// pub async fn create_schema_variant_with_root(
//     ctx: &DalContext,
//     schema_id: SchemaId,
// ) -> (SchemaVariant, RootProp) {
//     let name = generate_fake_name();
//     let (variant, root) = SchemaVariant::new(ctx, schema_id, &name, &name)
//         .await
//         .expect("cannot create schema variant");
//
//     let identity_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Identity)
//         .await
//         .expect("could not find identity func");
//
//     InputSocket::new(
//         ctx,
//         variant.id(),
//         "input",
//         identity_func_id,
//         SocketArity::Many,
//         SocketKind::Standard,
//         None,
//     )
//     .await
//     .expect("unable to create socket");
//
//     OutputSocket::new(
//         ctx,
//         variant.id(),
//         "output",
//         None,
//         identity_func_id,
//         SocketArity::Many,
//         SocketKind::Standard,
//         None,
//     )
//     .await
//     .expect("unable to create socket");
//
//     (variant, root)
// }

// pub async fn create_prop_without_ui_optionals(
//     ctx: &DalContext,
//     name: impl AsRef<str>,
//     kind: PropKind,
//     schema_variant_id: SchemaVariantId,
//     parent_prop_id: Option<PropId>,
// ) -> Prop {
//     Prop::new_without_ui_optionals(ctx, name, kind, schema_variant_id, parent_prop_id)
//         .await
//         .expect("could not create prop")
// }

// pub async fn create_component_and_schema(ctx: &DalContext) -> Component {
//     let schema = create_schema(ctx).await;
//     let mut schema_variant = create_schema_variant(ctx, *schema.id()).await;
//     schema_variant
//         .finalize(ctx, None)
//         .await
//         .expect("unable to finalize schema variant");
//     let name = generate_fake_name();
//     let (component, _) = Component::new(ctx, &name, *schema_variant.id())
//         .await
//         .expect("cannot create component");
//     component
// }

pub async fn create_component_for_schema_name(
    ctx: &DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Component {
    let schema = Schema::find_by_name(ctx, schema_name)
        .await
        .expect("could not find schema")
        .expect("schema not found");
    let schema_variant = SchemaVariant::list_for_schema(ctx, schema.id())
        .await
        .expect("failed listing schema variants")
        .pop()
        .expect("no schema variant found");

    Component::new(ctx, name.as_ref().to_string(), schema_variant.id())
        .await
        .expect("could not create component")
}

pub async fn create_component_for_schema_variant(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> Component {
    let name = generate_fake_name();
    Component::new(ctx, &name, schema_variant_id)
        .await
        .expect("cannot create component")
}

// pub async fn create_component_for_schema(ctx: &DalContext, schema_id: &SchemaId) -> Component {
//     let name = generate_fake_name();
//     let (component, _) = Component::new_for_default_variant_from_schema(ctx, &name, *schema_id)
//         .await
//         .expect("cannot create component");
//     component
// }

// pub async fn create_node(ctx: &DalContext, node_kind: &NodeKind) -> Node {
//     Node::new(ctx, node_kind).await.expect("cannot create node")
// }

// pub async fn create_func(ctx: &DalContext) -> Func {
//     let name = generate_fake_name();
//     Func::new(
//         ctx,
//         name,
//         FuncBackendKind::String,
//         FuncBackendResponseType::String,
//     )
//     .await
//     .expect("cannot create func")
// }

pub async fn connect_components_with_socket_names(
    ctx: &DalContext,
    source_component_id: ComponentId,
    output_socket_name: impl AsRef<str>,
    destination_component_id: ComponentId,
    input_socket_name: impl AsRef<str>,
) {
    let from_socket_id = {
        let sv_id = Component::schema_variant_id(ctx, source_component_id)
            .await
            .expect("find schema variant for source component");

        OutputSocket::find_with_name(ctx, output_socket_name, sv_id)
            .await
            .expect("perform find output socket")
            .expect("find output socket")
            .id()
    };

    let to_socket_id = {
        let sv_id = Component::schema_variant_id(ctx, destination_component_id)
            .await
            .expect("find schema variant for destination component");

        InputSocket::find_with_name(ctx, input_socket_name, sv_id)
            .await
            .expect("perform find input socket")
            .expect("find input socket")
            .id()
    };

    Component::connect(
        ctx,
        source_component_id,
        from_socket_id,
        destination_component_id,
        to_socket_id,
    )
    .await
    .expect("could not connect components");
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

#[derive(Serialize, Deserialize, Debug)]
pub struct PropEditorTestView {
    pub prop: PropertyEditorProp,
    pub value: PropertyEditorValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<HashMap<String, PropEditorTestView>>,
}
#[allow(dead_code)]
impl PropEditorTestView {
    fn get_view(&self, prop_path: &[&str]) -> Value {
        let mut value = serde_json::to_value(self).expect("convert UnifiedViewItem to json Value");

        // "root" is necessary for compatibility with other prop apis, but we skip it here
        for &prop_name in prop_path.iter().skip(1) {
            value = value
                .get("children")
                .expect("get children entry of PropEditorView")
                .get(prop_name)
                .expect("get child entry of PropEditorView")
                .clone();
        }

        value
    }

    pub fn get_prop(&self, prop_path: &[&str]) -> Value {
        let view = self.get_view(prop_path);
        view.get("prop").expect("get prop field of view").clone()
    }
    pub fn get_value(&self, prop_path: &[&str]) -> Value {
        let view = self.get_view(prop_path);
        view.get("value").expect("get prop field of view").clone()
    }

    pub async fn for_component_id(ctx: &DalContext, component_id: ComponentId) -> Self {
        let sv_id = Component::schema_variant_id(ctx, component_id)
            .await
            .expect("get schema variant from component");

        let PropertyEditorValues {
            root_value_id,
            values,
            child_values,
        } = PropertyEditorValues::assemble(ctx, component_id)
            .await
            .expect("assemble property editor values");

        let PropertyEditorSchema { props, .. } = PropertyEditorSchema::assemble(ctx, sv_id)
            .await
            .expect("assemble property editor schema");

        let root_view = {
            let value = values
                .get(&root_value_id)
                .expect("get property editor root value")
                .clone();

            let prop = props.get(&value.prop_id).expect("get property editor prop");

            PropEditorTestView {
                prop: prop.clone(),
                value,
                children: Self::property_editor_compile_children(
                    root_value_id,
                    &prop.kind,
                    &values,
                    &child_values,
                    &props,
                ),
            }
        };

        root_view
    }

    fn property_editor_compile_children(
        parent_value_id: PropertyEditorValueId,
        parent_prop_kind: &PropertyEditorPropKind,
        values: &HashMap<PropertyEditorValueId, PropertyEditorValue>,
        child_values: &HashMap<PropertyEditorValueId, Vec<PropertyEditorValueId>>,
        props: &HashMap<PropertyEditorPropId, PropertyEditorProp>,
    ) -> Option<HashMap<String, PropEditorTestView>> {
        let mut children = HashMap::new();

        for (index, child_id) in enumerate(
            child_values
                .get(&parent_value_id)
                .expect("get prop editor value children"),
        ) {
            let value = values
                .get(child_id)
                .expect("get property editor root value")
                .clone();

            let prop = props.get(&value.prop_id).expect("get property editor prop");

            let key = match parent_prop_kind {
                PropertyEditorPropKind::Array => index.to_string(),
                PropertyEditorPropKind::Map => value.key.clone().unwrap_or("ERROR".to_string()),
                _ => prop.name.clone(),
            };

            let child = PropEditorTestView {
                prop: prop.clone(),
                value,
                children: Self::property_editor_compile_children(
                    *child_id,
                    &prop.kind,
                    values,
                    child_values,
                    props,
                ),
            };

            children.insert(key, child);
        }

        if children.is_empty() {
            None
        } else {
            Some(children)
        }
    }
}
