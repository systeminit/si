use si_pkg::{
    SchemaVariantSpecPropRoot, SiPkg, SiPkgActionFunc, SiPkgAttrFuncInputView, SiPkgAuthFunc,
    SiPkgComponent, SiPkgEdge, SiPkgError, SiPkgFunc, SiPkgFuncArgument, SiPkgFuncData, SiPkgKind,
    SiPkgLeafFunction, SiPkgMetadata, SiPkgProp, SiPkgPropData, SiPkgSchema, SiPkgSchemaData,
    SiPkgSchemaVariant, SiPkgSocket, SiPkgSocketData, SocketSpecKind,
};
use std::{collections::HashMap, path::Path};
use telemetry::prelude::*;
use tokio::sync::Mutex;

use crate::attribute::prototype::argument::{
    value_source::ValueSource, AttributePrototypeArgument, AttributePrototypeArgumentId,
};
use crate::authentication_prototype::{AuthenticationPrototype, AuthenticationPrototypeId};
use crate::prop::PropParent;
use crate::{func::intrinsics::IntrinsicFunc, ComponentKind, SocketKind};
use crate::{
    func::{self, argument::FuncArgument},
    installed_pkg::{
        InstalledPkg, InstalledPkgAsset, InstalledPkgAssetKind, InstalledPkgAssetTyped,
        InstalledPkgId,
    },
    prop::PropPath,
    schema::variant::leaves::{LeafInputLocation, LeafKind},
    ActionPrototype, ChangeSetPk, DalContext, Func, FuncId, InputSocket, OutputSocket,
    OutputSocketId, Prop, PropId, PropKind, Schema, SchemaId, SchemaVariant, SchemaVariantId,
    StandardModel,
};
use crate::{AttributePrototype, AttributePrototypeId};

use super::{PkgError, PkgResult};

#[derive(Clone, Debug)]
pub(crate) enum Thing {
    ActionPrototype(ActionPrototype),
    AuthPrototype(AuthenticationPrototype),
    // AttributePrototypeArgument(AttributePrototypeArgument),
    // Component((Component, Node)),
    // Edge(Edge),
    Func(Func),
    #[allow(dead_code)]
    FuncArgument(FuncArgument),
    Schema(Schema),
    SchemaVariant(SchemaVariant),
    Socket(Box<(Option<InputSocket>, Option<OutputSocket>)>),
}

pub type ThingMap = super::ChangeSetThingMap<String, Thing>;

#[derive(Clone, Debug, Default)]
pub struct ImportOptions {
    pub schemas: Option<Vec<String>>,
    pub skip_import_funcs: Option<HashMap<String, Func>>,
    /// If set to `true`, the importer will install the assets from the module
    /// but will not make a record of the install as an "installed module".
    pub no_record: bool,
    /// If set to `true` then we will set the functions to a builtin
    /// in the UI. They will be marked as such.
    pub is_builtin: bool,
}

const SPECIAL_CASE_FUNCS: [&str; 2] = ["si:resourcePayloadToValue", "si:normalizeToArray"];

#[allow(clippy::too_many_arguments)]
async fn import_change_set(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    metadata: &SiPkgMetadata,
    funcs: &[SiPkgFunc<'_>],
    schemas: &[SiPkgSchema<'_>],
    _components: &[SiPkgComponent<'_>],
    _edges: &[SiPkgEdge<'_>],
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
    options: &ImportOptions,
) -> PkgResult<(
    Vec<SchemaVariantId>,
    Vec<(String, Vec<bool /*ImportAttributeSkip*/>)>,
    Vec<bool /*ImportEdgeSkip*/>,
)> {
    for func_spec in funcs {
        let unique_id = func_spec.unique_id().to_string();

        // This is a hack because the hash of the intrinsics has changed from the version in the
        // packages. We also apply this to si:resourcePayloadToValue since it should be an
        // intrinsic but is only in our packages
        if func::is_intrinsic(func_spec.name())
            || SPECIAL_CASE_FUNCS.contains(&func_spec.name())
            || func_spec.is_from_builtin().unwrap_or(false)
        {
            if let Some(func_id) = Func::find_by_name(ctx, func_spec.name()).await? {
                let func = Func::get_by_id(ctx, func_id).await?;

                thing_map.insert(
                    change_set_pk,
                    unique_id.to_owned(),
                    Thing::Func(func.to_owned()),
                );
            } else if let Some(func) = import_func(
                ctx,
                None,
                func_spec,
                installed_pkg_id,
                thing_map,
                options.is_builtin,
            )
            .await?
            {
                let args = func_spec.arguments()?;

                if !args.is_empty() {
                    import_func_arguments(ctx, None, func.id, &args, thing_map).await?;
                }
            }
        } else {
            let func = if let Some(Some(func)) = options
                .skip_import_funcs
                .as_ref()
                .map(|skip_funcs| skip_funcs.get(&unique_id))
            {
                if let Some(installed_pkg_id) = installed_pkg_id {
                    InstalledPkgAsset::new(
                        ctx,
                        InstalledPkgAssetTyped::new_for_func(
                            func.id,
                            installed_pkg_id,
                            func_spec.hash().to_string(),
                        ),
                    )
                    .await?;
                }

                // We're not going to import this func but we need it in the map for lookups later
                thing_map.insert(
                    change_set_pk,
                    func_spec.unique_id().to_owned(),
                    Thing::Func(func.to_owned()),
                );

                None
            } else {
                import_func(
                    ctx,
                    change_set_pk,
                    func_spec,
                    installed_pkg_id,
                    thing_map,
                    options.is_builtin,
                )
                .await?
            };

            if let Some(func) = func {
                thing_map.insert(
                    change_set_pk,
                    unique_id.to_owned(),
                    Thing::Func(func.to_owned()),
                );

                let args = func_spec.arguments()?;

                if !args.is_empty() {
                    import_func_arguments(ctx, change_set_pk, func.id, &args, thing_map).await?;
                }
            }
        };
    }

    let mut installed_schema_variant_ids = vec![];

    for schema_spec in schemas {
        match &options.schemas {
            None => {}
            Some(schemas) => {
                if !schemas.contains(&schema_spec.name().to_string().to_lowercase()) {
                    continue;
                }
            }
        }

        info!(
            "installing schema '{}' from {}",
            schema_spec.name(),
            metadata.name(),
        );

        let (_, schema_variant_ids) =
            import_schema(ctx, change_set_pk, schema_spec, installed_pkg_id, thing_map).await?;

        installed_schema_variant_ids.extend(schema_variant_ids);
    }

    // let mut component_attribute_skips = vec![];
    // for component_spec in components {
    //     let skips = import_component(ctx, change_set_pk, component_spec, thing_map).await?;
    //     if !skips.is_empty() {
    //         component_attribute_skips.push((component_spec.name().to_owned(), skips));
    //     }
    // }

    // let mut edge_skips = vec![];
    // for edge_spec in edges {
    //     if let Some(skip) = import_edge(ctx, change_set_pk, edge_spec, thing_map).await? {
    //         edge_skips.push(skip);
    //     }
    // }
    //

    Ok((
        installed_schema_variant_ids,
        vec![], // component_attribute_skips,
        vec![], // edge_skips,
    ))
}

// #[derive(Eq, PartialEq, Hash, Debug, Clone)]
// struct ValueCacheKey {
//     context: AttributeContext,
//     key: Option<String>,
//     index: Option<i64>,
// }

// impl ValueCacheKey {
//     pub fn new(
//         component_id: ComponentId,
//         prop_id: PropId,
//         key: Option<String>,
//         index: Option<i64>,
//     ) -> Self {
//         let mut context_builder = AttributeContextBuilder::new();
//         context_builder
//             .set_prop_id(prop_id)
//             .set_component_id(component_id);

//         Self {
//             context: context_builder.to_context_unchecked(),
//             key,
//             index,
//         }
//     }
// }

// async fn import_edge(
//     ctx: &DalContext,
//     change_set_pk: Option<ChangeSetPk>,
//     edge_spec: &SiPkgEdge<'_>,
//     thing_map: &mut ThingMap,
// ) -> PkgResult<Option<ImportEdgeSkip>> {
//     let edge = match thing_map.get(change_set_pk, &edge_spec.unique_id().to_owned()) {
//         Some(Thing::Edge(edge)) => Some(edge.to_owned()),
//         _ => {
//             if !edge_spec.deleted() {
//                 let head_component_unique_id = edge_spec.to_component_unique_id().to_owned();
//                 let (_, head_node) = match thing_map.get(change_set_pk, &head_component_unique_id) {
//                     Some(Thing::Component((component, node))) => (component, node),
//                     _ => {
//                         return Err(PkgError::MissingComponentForEdge(
//                             head_component_unique_id,
//                             edge_spec.from_socket_name().to_owned(),
//                             edge_spec.to_socket_name().to_owned(),
//                         ))
//                     }
//                 };

//                 let tail_component_unique_id = edge_spec.from_component_unique_id().to_owned();
//                 let (_, tail_node) = match thing_map.get(change_set_pk, &tail_component_unique_id) {
//                     Some(Thing::Component((component, node))) => (component, node),
//                     _ => {
//                         return Err(PkgError::MissingComponentForEdge(
//                             tail_component_unique_id,
//                             edge_spec.from_socket_name().to_owned(),
//                             edge_spec.to_socket_name().to_owned(),
//                         ))
//                     }
//                 };

//                 let to_socket = match Socket::find_by_name_for_edge_kind_and_node(
//                     ctx,
//                     edge_spec.to_socket_name(),
//                     SocketEdgeKind::ConfigurationInput,
//                     *head_node.id(),
//                 )
//                 .await?
//                 {
//                     Some(socket) => socket,
//                     None => {
//                         return Ok(Some(ImportEdgeSkip::MissingInputSocket(
//                             edge_spec.to_socket_name().to_owned(),
//                         )))
//                     }
//                 };

//                 let from_socket = match Socket::find_by_name_for_edge_kind_and_node(
//                     ctx,
//                     edge_spec.from_socket_name(),
//                     SocketEdgeKind::ConfigurationOutput,
//                     *tail_node.id(),
//                 )
//                 .await?
//                 {
//                     Some(socket) => socket,
//                     None => {
//                         return Ok(Some(ImportEdgeSkip::MissingOutputSocket(
//                             edge_spec.from_socket_name().to_owned(),
//                         )))
//                     }
//                 };

//                 Some(
//                     Edge::new_for_connection(
//                         ctx,
//                         *head_node.id(),
//                         *to_socket.id(),
//                         *tail_node.id(),
//                         *from_socket.id(),
//                         match edge_spec.edge_kind() {
//                             EdgeSpecKind::Configuration => EdgeKind::Configuration,
//                             EdgeSpecKind::Symbolic => EdgeKind::Symbolic,
//                         },
//                     )
//                     .await?,
//                 )
//             } else {
//                 None
//             }
//         }
//     };

//     if let Some(mut edge) = edge {
//         let creation_user_pk = match edge_spec.creation_user_pk() {
//             Some(pk_str) => Some(UserPk::from_str(pk_str)?),
//             None => None,
//         };
//         if creation_user_pk.as_ref() != edge.creation_user_pk() {
//             edge.set_creation_user_pk(ctx, creation_user_pk).await?;
//         }

//         let deletion_user_pk = match edge_spec.deletion_user_pk() {
//             Some(pk_str) => Some(UserPk::from_str(pk_str)?),
//             None => None,
//         };

//         if deletion_user_pk.as_ref() != edge.deletion_user_pk() {
//             edge.set_deletion_user_pk(ctx, deletion_user_pk).await?;
//         }

//         if edge.deleted_implicitly() != edge_spec.deleted_implicitly() {
//             edge.set_deleted_implicitly(ctx, edge_spec.deleted_implicitly())
//                 .await?;
//         }

//         if edge.visibility().is_deleted() && !edge_spec.deleted() {
//             Edge::restore_by_id(ctx, *edge.id()).await?;
//         } else if !edge.visibility().is_deleted() && edge_spec.deleted() {
//             edge.delete_and_propagate(ctx).await?;
//         }

//         thing_map.insert(
//             change_set_pk,
//             edge_spec.unique_id().to_owned(),
//             Thing::Edge(edge),
//         );
//     }

//     Ok(None)
// }

// async fn import_component(
//     ctx: &DalContext,
//     change_set_pk: Option<ChangeSetPk>,
//     component_spec: &SiPkgComponent<'_>,
//     thing_map: &mut ThingMap,
// ) -> PkgResult<Vec<ImportAttributeSkip>> {
//     let _change_set_pk_inner = change_set_pk.ok_or(PkgError::ComponentImportWithoutChangeSet)?;

//     let variant = match component_spec.variant() {
//         ComponentSpecVariant::BuiltinVariant {
//             schema_name,
//             variant_name,
//         } => {
//             let schema = Schema::find_by_name_builtin(ctx, schema_name.as_str())
//                 .await?
//                 .ok_or(PkgError::ComponentMissingBuiltinSchema(
//                     schema_name.to_owned(),
//                     component_spec.name().into(),
//                 ))?;

//             schema
//                 .find_variant_by_name(ctx, variant_name.as_str())
//                 .await?
//                 .ok_or(PkgError::ComponentMissingBuiltinSchemaVariant(
//                     schema_name.to_owned(),
//                     variant_name.to_owned(),
//                     component_spec.name().into(),
//                 ))?
//         }
//         ComponentSpecVariant::WorkspaceVariant { variant_unique_id } => {
//             match thing_map.get(change_set_pk, variant_unique_id) {
//                 Some(Thing::SchemaVariant(variant)) => variant.to_owned(),
//                 _ => {
//                     return Err(PkgError::ComponentMissingSchemaVariant(
//                         variant_unique_id.to_owned(),
//                         component_spec.name().into(),
//                     ))
//                 }
//             }
//         }
//     };

//     let (mut component, mut node) =
//         match thing_map.get(change_set_pk, &component_spec.unique_id().to_owned()) {
//             Some(Thing::Component((existing_component, node))) => {
//                 (existing_component.to_owned(), node.to_owned())
//             }
//             _ => {
//                 let (component, node) =
//                     Component::new(ctx, component_spec.name(), *variant.id()).await?;
//                 thing_map.insert(
//                     change_set_pk,
//                     component_spec.unique_id().into(),
//                     Thing::Component((component.to_owned(), node.to_owned())),
//                 );

//                 (component, node)
//             }
//         };

//     if component.name(ctx).await? != component_spec.name() {
//         component.set_name(ctx, Some(component_spec.name())).await?;
//     }

//     let position = component_spec
//         .position()?
//         .pop()
//         .ok_or(PkgError::ComponentSpecMissingPosition)?;

//     if node.x() != position.x() {
//         node.set_x(ctx, position.x()).await?;
//     }
//     if node.y() != position.y() {
//         node.set_y(ctx, position.y()).await?;
//     }

//     if node.height() != position.height() {
//         node.set_height(ctx, position.height().map(ToOwned::to_owned))
//             .await?;
//     }
//     if node.width() != position.width() {
//         node.set_width(ctx, position.width().map(ToOwned::to_owned))
//             .await?;
//     }

//     let mut value_cache: HashMap<ValueCacheKey, AttributeValue> = HashMap::new();
//     let mut prop_cache: HashMap<String, Option<Prop>> = HashMap::new();

//     let mut skips = vec![];

//     for attribute in component_spec.attributes()? {
//         if let Some(skip) = import_component_attribute(
//             ctx,
//             change_set_pk,
//             &component,
//             &variant,
//             attribute,
//             &mut value_cache,
//             &mut prop_cache,
//             thing_map,
//         )
//         .await?
//         {
//             skips.push(skip);
//         }
//     }
//     for attribute in component_spec.input_sockets()? {
//         if let Some(skip) = import_component_attribute(
//             ctx,
//             change_set_pk,
//             &component,
//             &variant,
//             attribute,
//             &mut value_cache,
//             &mut prop_cache,
//             thing_map,
//         )
//         .await?
//         {
//             skips.push(skip);
//         }
//     }
//     for attribute in component_spec.output_sockets()? {
//         if let Some(skip) = import_component_attribute(
//             ctx,
//             change_set_pk,
//             &component,
//             &variant,
//             attribute,
//             &mut value_cache,
//             &mut prop_cache,
//             thing_map,
//         )
//         .await?
//         {
//             skips.push(skip);
//         }
//     }

//     if component_spec.needs_destroy() {
//         component.set_needs_destroy(ctx, true).await?;
//     }

//     if component.visibility().is_deleted() && !component_spec.deleted() {
//         Component::restore_and_propagate(ctx, *component.id()).await?;
//     } else if !component.visibility().is_deleted() && component_spec.deleted() {
//         component.delete_and_propagate(ctx).await?;
//     }

//     Ok(skips)
// }

// fn get_prop_kind_for_value(value: Option<&serde_json::Value>) -> Option<PropKind> {
//     match value {
//         Some(serde_json::Value::Array(_)) => Some(PropKind::Array),
//         Some(serde_json::Value::Bool(_)) => Some(PropKind::Boolean),
//         Some(serde_json::Value::Number(_)) => Some(PropKind::Integer),
//         Some(serde_json::Value::Object(_)) => Some(PropKind::Object),
//         Some(serde_json::Value::String(_)) => Some(PropKind::String),

//         _ => None,
//     }
// }

// #[allow(clippy::too_many_arguments)]
// async fn import_component_attribute(
//     ctx: &DalContext,
//     change_set_pk: Option<ChangeSetPk>,
//     component: &Component,
//     variant: &SchemaVariant,
//     attribute: &SiPkgAttributeValue<'_>,
//     value_cache: &mut HashMap<ValueCacheKey, AttributeValue>,
//     prop_cache: &mut HashMap<String, Option<Prop>>,
//     thing_map: &mut ThingMap,
// ) -> PkgResult<Option<ImportAttributeSkip>> {
//     match attribute.path() {
//         AttributeValuePath::Prop { path, key, index } => {
//             if attribute.parent_path().is_none() && (key.is_some() || index.is_some()) {
//                 return Err(PkgError::AttributeValueWithKeyOrIndexButNoParent);
//             }

//             let prop = match prop_cache.get(path) {
//                 Some(prop) => prop.to_owned(),
//                 None => {
//                     let prop = Prop::find_prop_by_path_opt(
//                         ctx,
//                         *variant.id(),
//                         &PropPath::from(path.to_owned()),
//                     )
//                     .await?;
//                     prop_cache.insert(path.to_owned(), prop.to_owned());

//                     prop
//                 }
//             };

//             struct ParentData<'a> {
//                 prop: Option<&'a Prop>,
//                 attribute_value: Option<AttributeValue>,
//                 default_attribute_value: Option<AttributeValue>,
//             }

//             match prop {
//                 Some(prop) => {
//                     // Validate type if possible
//                     let expected_prop_kind = get_prop_kind_for_value(attribute.value());
//                     if let Some(expected_kind) = expected_prop_kind {
//                         if expected_kind
//                             != match prop.kind() {
//                                 PropKind::Map | PropKind::Object => PropKind::Object,
//                                 other => *other,
//                             }
//                         {
//                             return Ok(Some(ImportAttributeSkip::KindMismatch {
//                                 path: PropPath::from(path),
//                                 expected_kind,
//                                 variant_kind: *prop.kind(),
//                             }));
//                         }
//                     }

//                     let parent_data = if let Some(AttributeValuePath::Prop { path, key, index }) =
//                         attribute.parent_path()
//                     {
//                         let parent_prop = prop_cache
//                             .get(path)
//                             .and_then(|p| p.as_ref())
//                             .ok_or(PkgError::AttributeValueParentPropNotFound(path.to_owned()))?;

//                         let parent_value_cache_key = ValueCacheKey::new(
//                             *component.id(),
//                             *parent_prop.id(),
//                             key.to_owned(),
//                             index.to_owned(),
//                         );

//                         let parent_av = value_cache.get(&parent_value_cache_key).ok_or(
//                             PkgError::AttributeValueParentValueNotFound(
//                                 path.to_owned(),
//                                 key.to_owned(),
//                                 index.to_owned(),
//                             ),
//                         )?;

//                         let parent_default_value_cache_key = ValueCacheKey::new(
//                             ComponentId::NONE,
//                             *parent_prop.id(),
//                             key.to_owned(),
//                             index.to_owned(),
//                         );

//                         let parent_default_av =
//                             value_cache.get(&parent_default_value_cache_key).cloned();

//                         ParentData {
//                             prop: Some(parent_prop),
//                             attribute_value: Some(parent_av.to_owned()),
//                             default_attribute_value: parent_default_av,
//                         }
//                     } else {
//                         ParentData {
//                             prop: None,
//                             attribute_value: None,
//                             default_attribute_value: None,
//                         }
//                     };

//                     // If we're an array element, we might already exist in the index map
//                     let av_id_from_index_map = match index {
//                         Some(index) => match parent_data.attribute_value.as_ref() {
//                             Some(parent_av) => {
//                                 match parent_av
//                                     .index_map()
//                                     .and_then(|index_map| index_map.order().get(*index as usize))
//                                 {
//                                     None => {
//                                         let attribute_context = AttributeContext::builder()
//                                             .set_prop_id(*prop.id())
//                                             .set_component_id(*component.id())
//                                             .to_context_unchecked();

//                                         // This value will get updated by
//                                         // update_attribute_value
//                                         Some(
//                                             AttributeValue::insert_for_context(
//                                                 ctx,
//                                                 attribute_context,
//                                                 *parent_av.id(),
//                                                 None,
//                                                 None,
//                                             )
//                                             .await?,
//                                         )
//                                     }
//                                     Some(av_id) => Some(*av_id),
//                                 }
//                             }
//                             None => None,
//                         },
//                         None => None,
//                     };

//                     let default_value_cache_key = ValueCacheKey::new(
//                         ComponentId::NONE,
//                         *prop.id(),
//                         key.to_owned(),
//                         index.to_owned(),
//                     );

//                     let default_av = match value_cache.entry(default_value_cache_key) {
//                         Entry::Occupied(occupied) => Some(occupied.get().to_owned()),
//                         Entry::Vacant(vacant) => {
//                             if parent_data.default_attribute_value.is_none()
//                                 && parent_data.prop.is_some()
//                             {
//                                 None
//                             } else {
//                                 let default_parent_av_id =
//                                     parent_data.default_attribute_value.map(|av| *av.id());

//                                 let default_value_context = AttributeReadContext {
//                                     prop_id: Some(*prop.id()),
//                                     internal_provider_id: Some(InternalProviderId::NONE),
//                                     external_provider_id: Some(ExternalProviderId::NONE),
//                                     component_id: None,
//                                 };

//                                 let value = AttributeValue::find_with_parent_and_key_for_context(
//                                     ctx,
//                                     default_parent_av_id,
//                                     key.to_owned(),
//                                     default_value_context,
//                                 )
//                                 .await?;

//                                 if let Some(value) = &value {
//                                     vacant.insert(value.to_owned());
//                                 }

//                                 value
//                             }
//                         }
//                     };

//                     let context = AttributeReadContext {
//                         prop_id: Some(*prop.id()),
//                         internal_provider_id: Some(InternalProviderId::NONE),
//                         external_provider_id: Some(ExternalProviderId::NONE),
//                         component_id: Some(*component.id()),
//                     };

//                     let parent_av_id = parent_data.attribute_value.as_ref().map(|av| *av.id());
//                     let maybe_av = match av_id_from_index_map {
//                         Some(av_id) => Some(AttributeValue::get_by_id(ctx, &av_id).await?.ok_or(
//                             AttributeValueError::NotFound(av_id, ctx.visibility().to_owned()),
//                         )?),
//                         None => {
//                             AttributeValue::find_with_parent_and_key_for_context(
//                                 ctx,
//                                 parent_av_id,
//                                 key.to_owned(),
//                                 context,
//                             )
//                             .await?
//                         }
//                     };

//                     let mut av_to_update = match maybe_av {
//                         Some(av) => av,
//                         None => {
//                             if index.is_some() {
//                                 dbg!(
//                                     "should always have an attribute value here for an indexed av"
//                                 );
//                             }
//                             let context = AttributeReadContext {
//                                 prop_id: Some(*prop.id()),
//                                 internal_provider_id: None,
//                                 external_provider_id: None,
//                                 component_id: None,
//                             };
//                             let maybe_av = AttributeValue::find_with_parent_and_key_for_context(
//                                 ctx,
//                                 parent_av_id,
//                                 key.to_owned(),
//                                 context,
//                             )
//                             .await?;

//                             match maybe_av {
//                                 Some(av) => av,
//                                 None => {
//                                     let parent_av_id = parent_av_id.ok_or(
//                                         PkgError::AttributeValueParentValueNotFound(
//                                             "in av search".into(),
//                                             key.to_owned(),
//                                             index.to_owned(),
//                                         ),
//                                     )?;

//                                     let attribute_context = AttributeContext::builder()
//                                         .set_prop_id(*prop.id())
//                                         .set_component_id(*component.id())
//                                         .to_context_unchecked();

//                                     if key.is_some() {
//                                         let av_id = AttributeValue::insert_for_context(
//                                             ctx,
//                                             attribute_context,
//                                             parent_av_id,
//                                             None,
//                                             key.to_owned(),
//                                         )
//                                         .await?;

//                                         AttributeValue::get_by_id(ctx, &av_id).await?.ok_or(
//                                             AttributeValueError::NotFound(
//                                                 av_id,
//                                                 ctx.visibility().to_owned(),
//                                             ),
//                                         )?
//                                     } else {
//                                         let (_, value) = create_attribute_value(
//                                             ctx,
//                                             change_set_pk,
//                                             attribute_context,
//                                             *component.id(),
//                                             key,
//                                             parent_data.attribute_value.as_ref(),
//                                             default_av.as_ref(),
//                                             &attribute,
//                                             thing_map,
//                                         )
//                                         .await?;

//                                         value
//                                     }
//                                 }
//                             }
//                         }
//                     };

//                     let updated_av = update_attribute_value(
//                         ctx,
//                         change_set_pk,
//                         *variant.id(),
//                         *component.id(),
//                         &attribute,
//                         &mut av_to_update,
//                         parent_data.attribute_value.as_ref(),
//                         default_av.as_ref(),
//                         thing_map,
//                     )
//                     .await?;

//                     let this_cache_key = ValueCacheKey::new(
//                         *component.id(),
//                         *prop.id(),
//                         key.to_owned(),
//                         index.to_owned(),
//                     );

//                     value_cache.insert(this_cache_key, updated_av);
//                 }
//                 None => {
//                     // collect missing props and log them
//                     return Ok(Some(ImportAttributeSkip::MissingProp(PropPath::from(path))));
//                 }
//             }
//         }
//         AttributeValuePath::InputSocket(socket_name)
//         | AttributeValuePath::OutputSocket(socket_name) => {
//             let (default_read_context, read_context, write_context) =
//                 if matches!(attribute.path(), AttributeValuePath::InputSocket(_)) {
//                     let internal_provider =
//                         match InternalProvider::find_explicit_for_schema_variant_and_name(
//                             ctx,
//                             *variant.id(),
//                             socket_name.as_str(),
//                         )
//                         .await?
//                         {
//                             None => {
//                                 return Ok(Some(ImportAttributeSkip::MissingInputSocket(
//                                     socket_name.to_owned(),
//                                 )))
//                             }
//                             Some(ip) => ip,
//                         };

//                     let default_read_context = AttributeReadContext {
//                         prop_id: Some(PropId::NONE),
//                         internal_provider_id: Some(*internal_provider.id()),
//                         external_provider_id: Some(ExternalProviderId::NONE),
//                         component_id: None,
//                     };
//                     let read_context = AttributeReadContext {
//                         prop_id: Some(PropId::NONE),
//                         internal_provider_id: Some(*internal_provider.id()),
//                         external_provider_id: Some(ExternalProviderId::NONE),
//                         component_id: Some(*component.id()),
//                     };
//                     let write_context = AttributeContext::builder()
//                         .set_internal_provider_id(*internal_provider.id())
//                         .set_component_id(*component.id())
//                         .to_context_unchecked();

//                     (default_read_context, read_context, write_context)
//                 } else {
//                     let external_provider =
//                         match ExternalProvider::find_for_schema_variant_and_name(
//                             ctx,
//                             *variant.id(),
//                             socket_name.as_str(),
//                         )
//                         .await?
//                         {
//                             None => {
//                                 return Ok(Some(ImportAttributeSkip::MissingOutputSocket(
//                                     socket_name.to_owned(),
//                                 )))
//                             }
//                             Some(ep) => ep,
//                         };

//                     let default_read_context = AttributeReadContext {
//                         prop_id: Some(PropId::NONE),
//                         internal_provider_id: Some(InternalProviderId::NONE),
//                         external_provider_id: Some(*external_provider.id()),
//                         component_id: None,
//                     };
//                     let read_context = AttributeReadContext {
//                         prop_id: Some(PropId::NONE),
//                         internal_provider_id: Some(InternalProviderId::NONE),
//                         external_provider_id: Some(*external_provider.id()),
//                         component_id: Some(*component.id()),
//                     };
//                     let write_context = AttributeContext::builder()
//                         .set_external_provider_id(*external_provider.id())
//                         .set_component_id(*component.id())
//                         .to_context_unchecked();

//                     (default_read_context, read_context, write_context)
//                 };

//             let default_value = AttributeValue::find_for_context(ctx, default_read_context).await?;

//             match AttributeValue::find_for_context(ctx, read_context).await? {
//                 Some(mut existing_av) => {
//                     update_attribute_value(
//                         ctx,
//                         change_set_pk,
//                         *variant.id(),
//                         *component.id(),
//                         &attribute,
//                         &mut existing_av,
//                         None,
//                         default_value.as_ref(),
//                         thing_map,
//                     )
//                     .await?;
//                 }
//                 None => {
//                     create_attribute_value(
//                         ctx,
//                         change_set_pk,
//                         write_context,
//                         *component.id(),
//                         &None,
//                         None,
//                         default_value.as_ref(),
//                         &attribute,
//                         thing_map,
//                     )
//                     .await?;
//                 }
//             }
//         }
//     }

//     Ok(None)
// }

// async fn get_ip_for_input(
//     ctx: &DalContext,
//     schema_variant_id: SchemaVariantId,
//     input: &SiPkgAttrFuncInput<'_>,
// ) -> PkgResult<Option<InternalProviderId>> {
//     Ok(match input {
//         SiPkgAttrFuncInput::Prop { prop_path, .. } => {
//             let input_source_prop = match Prop::find_prop_by_path_opt(
//                 ctx,
//                 schema_variant_id,
//                 &PropPath::from(prop_path),
//             )
//             .await?
//             {
//                 Some(p) => p,
//                 None => return Ok(None),
//             };

//             let ip = InternalProvider::find_for_prop(ctx, *input_source_prop.id())
//                 .await?
//                 .ok_or(PkgError::MissingInternalProviderForProp(
//                     *input_source_prop.id(),
//                 ))?;

//             Some(*ip.id())
//         }
//         SiPkgAttrFuncInput::InputSocket { socket_name, .. } => {
//             let explicit_ip = match InternalProvider::find_explicit_for_schema_variant_and_name(
//                 ctx,
//                 schema_variant_id,
//                 &socket_name,
//             )
//             .await?
//             {
//                 Some(ip) => ip,
//                 None => return Ok(None),
//             };

//             Some(*explicit_ip.id())
//         }
//         SiPkgAttrFuncInput::OutputSocket { .. } => None,
//     })
// }

// #[allow(clippy::too_many_arguments)]
// async fn create_attribute_value(
//     ctx: &DalContext,
//     change_set_pk: Option<ChangeSetPk>,
//     context: AttributeContext,
//     component_id: ComponentId,
//     real_key: &Option<String>,
//     parent_attribute_value: Option<&AttributeValue>,
//     default_attribute_value: Option<&AttributeValue>,
//     attribute_spec: &SiPkgAttributeValue<'_>,
//     thing_map: &mut ThingMap,
// ) -> PkgResult<(AttributePrototype, AttributeValue)> {
//     let attribute_func =
//         match thing_map.get(change_set_pk, &attribute_spec.func_unique_id().to_owned()) {
//             Some(Thing::Func(func)) => func,
//             _ => {
//                 return Err(PkgError::MissingFuncUniqueId(format!(
//                     "here, {}",
//                     attribute_spec.func_unique_id().to_owned()
//                 )));
//             }
//         };

//     let new_context = AttributeContext::builder()
//         .set_prop_id(context.prop_id())
//         .set_internal_provider_id(context.internal_provider_id())
//         .set_external_provider_id(context.external_provider_id())
//         .set_component_id(component_id)
//         .to_context_unchecked();

//     let func_binding = FuncBinding::new(
//         ctx,
//         attribute_spec.func_binding_args().to_owned(),
//         *attribute_func.id(),
//         attribute_spec.backend_kind().into(),
//     )
//     .await?;

//     let mut func_binding_return_value = FuncBindingReturnValue::new(
//         ctx,
//         attribute_spec.unprocessed_value().cloned(),
//         attribute_spec.value().cloned(),
//         *attribute_func.id(),
//         *func_binding.id(),
//         FuncExecutionPk::NONE,
//     )
//     .await?;

//     let execution = FuncExecution::new(ctx, attribute_func, &func_binding).await?;
//     // TODO: add output stream?

//     func_binding_return_value
//         .set_func_execution_pk(ctx, execution.pk())
//         .await?;

//     let mut new_value = AttributeValue::new(
//         ctx,
//         *func_binding.id(),
//         *func_binding_return_value.id(),
//         new_context,
//         real_key.to_owned(),
//     )
//     .await?;

//     if let Some(parent_attribute_value) = parent_attribute_value.as_ref() {
//         new_value
//             .set_parent_attribute_value_unchecked(ctx, parent_attribute_value.id())
//             .await?;
//     }

//     if attribute_spec.is_proxy() {
//         let default_av =
//             default_attribute_value.ok_or(PkgError::AttributeValueSetToProxyButNoProxyFound)?;

//         new_value
//             .set_proxy_for_attribute_value_id(ctx, Some(*default_av.id()))
//             .await?;
//     }

//     let prototype_context = AttributeContext::builder()
//         .set_prop_id(new_context.prop_id())
//         .set_external_provider_id(new_context.external_provider_id())
//         .set_internal_provider_id(new_context.internal_provider_id())
//         .set_component_id(if attribute_spec.component_specific() {
//             new_context.component_id()
//         } else {
//             ComponentId::NONE
//         })
//         .to_context_unchecked();

//     let prototype =
//         match AttributePrototype::find_for_context_and_key(ctx, prototype_context, real_key)
//             .await?
//             .pop()
//         {
//             Some(existing_proto) => {
//                 new_value
//                     .set_attribute_prototype(ctx, existing_proto.id())
//                     .await?;

//                 existing_proto
//             }
//             None => {
//                 AttributePrototype::new_with_existing_value(
//                     ctx,
//                     *attribute_func.id(),
//                     new_context,
//                     real_key.to_owned(),
//                     parent_attribute_value.map(|pav| *pav.id()),
//                     *new_value.id(),
//                 )
//                 .await?
//             }
//         };

//     Ok((prototype, new_value))
// }

// #[allow(clippy::too_many_arguments)]
// async fn update_attribute_value(
//     ctx: &DalContext,
//     change_set_pk: Option<ChangeSetPk>,
//     schema_variant_id: SchemaVariantId,
//     component_id: ComponentId,
//     attribute_spec: &SiPkgAttributeValue<'_>,
//     attribute_value: &mut AttributeValue,
//     parent_attribute_value: Option<&AttributeValue>,
//     default_attribute_value: Option<&AttributeValue>,
//     thing_map: &mut ThingMap,
// ) -> PkgResult<AttributeValue> {
//     let prototype = attribute_value
//         .attribute_prototype(ctx)
//         .await?
//         .ok_or(AttributeValueError::MissingAttributePrototype)?;

//     let attribute_func =
//         match thing_map.get(change_set_pk, &attribute_spec.func_unique_id().to_owned()) {
//             Some(Thing::Func(func)) => func,
//             _ => {
//                 return Err(PkgError::MissingFuncUniqueId(format!(
//                     "here, {}",
//                     attribute_spec.func_unique_id().to_owned()
//                 )));
//             }
//         };

//     let (mut prototype, value) = if prototype.context.component_id().is_none()
//         && attribute_spec.component_specific()
//     {
//         let current_context = attribute_value.context;
//         let new_context = AttributeContext::builder()
//             .set_prop_id(current_context.prop_id())
//             .set_internal_provider_id(current_context.internal_provider_id())
//             .set_external_provider_id(current_context.external_provider_id())
//             .set_component_id(component_id)
//             .to_context_unchecked();

//         let func_binding = FuncBinding::new(
//             ctx,
//             attribute_spec.func_binding_args().to_owned(),
//             *attribute_func.id(),
//             attribute_spec.backend_kind().into(),
//         )
//         .await?;

//         let mut func_binding_return_value = FuncBindingReturnValue::new(
//             ctx,
//             attribute_spec.unprocessed_value().cloned(),
//             attribute_spec.value().cloned(),
//             *attribute_func.id(),
//             *func_binding.id(),
//             FuncExecutionPk::NONE,
//         )
//         .await?;

//         let execution = FuncExecution::new(ctx, attribute_func, &func_binding).await?;
//         // TODO: add output stream?

//         func_binding_return_value
//             .set_func_execution_pk(ctx, execution.pk())
//             .await?;

//         let mut new_value = AttributeValue::new(
//             ctx,
//             *func_binding.id(),
//             *func_binding_return_value.id(),
//             new_context,
//             attribute_value.key(),
//         )
//         .await?;

//         if attribute_spec.is_proxy() {
//             let default_av =
//                 default_attribute_value.ok_or(PkgError::AttributeValueSetToProxyButNoProxyFound)?;

//             new_value
//                 .set_proxy_for_attribute_value_id(ctx, Some(*default_av.id()))
//                 .await?;
//         }

//         (
//             AttributePrototype::new_with_existing_value(
//                 ctx,
//                 *attribute_func.id(),
//                 new_context,
//                 attribute_value.key().map(|k| k.to_owned()),
//                 parent_attribute_value.map(|pav| *pav.id()),
//                 *new_value.id(),
//             )
//             .await?,
//             new_value,
//         )
//     } else {
//         let current_fb = FuncBinding::get_by_id(ctx, &attribute_value.func_binding_id())
//             .await?
//             .ok_or(FuncBindingError::NotFound(
//                 attribute_value.func_binding_id(),
//             ))?;

//         let current_fbrv =
//             FuncBindingReturnValue::get_by_id(ctx, &attribute_value.func_binding_return_value_id())
//                 .await?
//                 .ok_or(FuncBindingReturnValueError::NotFound(
//                     attribute_value.func_binding_return_value_id(),
//                 ))?;

//         if current_fb.args() != attribute_spec.func_binding_args()
//             || current_fbrv.unprocessed_value() != attribute_spec.unprocessed_value()
//             || current_fbrv.func_id() != attribute_func.id()
//             || current_fb.code_sha256() != attribute_func.code_sha256()
//         {
//             let func_binding = FuncBinding::new(
//                 ctx,
//                 attribute_spec.func_binding_args().to_owned(),
//                 *attribute_func.id(),
//                 attribute_spec.backend_kind().into(),
//             )
//             .await?;

//             let mut func_binding_return_value = FuncBindingReturnValue::new(
//                 ctx,
//                 attribute_spec.unprocessed_value().cloned(),
//                 attribute_spec.value().cloned(),
//                 *attribute_func.id(),
//                 *func_binding.id(),
//                 FuncExecutionPk::NONE,
//             )
//             .await?;

//             let execution = FuncExecution::new(ctx, attribute_func, &func_binding).await?;
//             // TODO: add output stream?

//             func_binding_return_value
//                 .set_func_execution_pk(ctx, execution.pk())
//                 .await?;

//             attribute_value
//                 .set_func_binding_id(ctx, *func_binding.id())
//                 .await?;

//             attribute_value
//                 .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
//                 .await?;
//         }

//         (prototype, attribute_value.to_owned())
//     };

//     if prototype.func_id() != *attribute_func.id() {
//         prototype.set_func_id(ctx, attribute_func.id()).await?;
//     }

//     let inputs = attribute_spec.inputs()?;

//     let mut current_apas =
//         AttributePrototypeArgument::list_for_attribute_prototype(ctx, *prototype.id()).await?;

//     if inputs.is_empty() && !current_apas.is_empty() {
//         for apa in current_apas.iter_mut() {
//             apa.delete_by_id(ctx).await?;
//         }
//     } else if !inputs.is_empty() {
//         let mut processed_inputs = HashSet::new();
//         for apa in current_apas.iter_mut() {
//             let func_arg = FuncArgument::get_by_id(ctx, &apa.func_argument_id())
//                 .await?
//                 .ok_or(PkgError::MissingFuncArgumentById(apa.func_argument_id()))?;

//             let matching_input = inputs.iter().find(|input| input.name() == func_arg.name());

//             match matching_input {
//                 Some(input) => {
//                     if let Some(ip_id) = get_ip_for_input(ctx, schema_variant_id, input).await? {
//                         if apa.internal_provider_id() != ip_id {
//                             apa.set_internal_provider_id(ctx, ip_id).await?;
//                         }
//                     }

//                     processed_inputs.insert(input.name());
//                 }
//                 None => apa.delete_by_id(ctx).await?,
//             }
//         }

//         for input in &inputs {
//             let name = input.name();

//             if processed_inputs.contains(name) {
//                 continue;
//             }

//             let func_arg = FuncArgument::find_by_name_for_func(ctx, name, *attribute_func.id())
//                 .await?
//                 .ok_or(PkgError::MissingFuncArgument(
//                     name.into(),
//                     *attribute_func.id(),
//                 ))?;

//             if let Some(ip_id) = get_ip_for_input(ctx, schema_variant_id, input).await? {
//                 AttributePrototypeArgument::new_for_intra_component(
//                     ctx,
//                     *prototype.id(),
//                     *func_arg.id(),
//                     ip_id,
//                 )
//                 .await?;
//             }
//         }
//     }

//     Ok(value)
// }

// #[derive(Debug, Clone, Deserialize, Serialize)]
// #[serde(rename_all = "camelCase")]
// pub struct ImportSkips {
//     change_set_pk: ChangeSetPk,
//     edge_skips: Vec<ImportEdgeSkip>,
//     attribute_skips: Vec<(String, Vec<ImportAttributeSkip>)>,
// }

// #[remain::sorted]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(tag = "type", rename_all = "camelCase")]
// pub enum ImportAttributeSkip {
//     #[serde(rename_all = "camelCase")]
//     KindMismatch {
//         path: PropPath,
//         expected_kind: PropKind,
//         variant_kind: PropKind,
//     },
//     MissingInputSocket(String),
//     MissingOutputSocket(String),
//     MissingProp(PropPath),
// }

// #[derive(Clone, Debug, Deserialize, Serialize)]
// #[serde(tag = "type", rename_all = "camelCase")]
// pub enum ImportEdgeSkip {
//     MissingInputSocket(String),
//     MissingOutputSocket(String),
// }

pub async fn import_pkg_from_pkg(
    ctx: &DalContext,
    pkg: &SiPkg,
    options: Option<ImportOptions>,
) -> PkgResult<(
    Option<InstalledPkgId>,
    Vec<SchemaVariantId>,
    Option<Vec<bool /*ImportSkips*/>>,
)> {
    // We have to write the installed_pkg row first, so that we have an id, and rely on transaction
    // semantics to remove the row if anything in the installation process fails
    let root_hash = pkg.hash()?.to_string();

    let options = options.unwrap_or_default();

    if InstalledPkg::find_by_hash(ctx, &root_hash).await?.is_some() {
        return Err(PkgError::PackageAlreadyInstalled(root_hash));
    }

    let metadata = pkg.metadata()?;

    let installed_pkg_id = if options.no_record {
        None
    } else {
        Some(
            *InstalledPkg::new(ctx, metadata.name(), pkg.hash()?.to_string())
                .await?
                .id(),
        )
    };

    let mut change_set_things = ThingMap::new();

    match metadata.kind() {
        SiPkgKind::Module => {
            let (installed_schema_variant_ids, _, _) = import_change_set(
                ctx,
                None,
                &metadata,
                &pkg.funcs()?,
                &pkg.schemas()?,
                &[],
                &[],
                installed_pkg_id,
                &mut change_set_things,
                &options,
            )
            .await?;

            Ok((installed_pkg_id, installed_schema_variant_ids, None))
        }
        SiPkgKind::WorkspaceBackup => {
            // let mut ctx = ctx.clone_with_new_visibility(ctx.visibility().to_head());

            // let mut import_skips = vec![];

            // let workspace_pk = WorkspacePk::from_str(
            //     metadata
            //         .workspace_pk()
            //         .ok_or(PkgError::WorkspacePkNotInBackup)?,
            // )?;
            // let workspace_name = metadata
            //     .workspace_name()
            //     .ok_or(PkgError::WorkspaceNameNotInBackup)?;
            // let default_change_set_name = metadata.default_change_set().unwrap_or("head");

            // Workspace::clear_or_create_workspace(&mut ctx, workspace_pk, workspace_name).await?;

            // ctx.update_tenancy(Tenancy::new(workspace_pk));

            // let change_sets = pkg.change_sets()?;
            // let default_change_set = change_sets
            //     .iter()
            //     .find(|cs| cs.name() == default_change_set_name)
            //     .ok_or(PkgError::WorkspaceBackupNoDefaultChangeSet(
            //         default_change_set_name.into(),
            //     ))?;

            // let (_, attribute_skips, edge_skips) = import_change_set(
            //     &ctx,
            //     Some(ChangeSetPk::NONE),
            //     &metadata,
            //     &default_change_set.funcs()?,
            //     &default_change_set.schemas()?,
            //     &default_change_set.components()?,
            //     &default_change_set.edges()?,
            //     installed_pkg_id,
            //     &mut change_set_things,
            //     &options,
            // )
            // .await?;

            // import_skips.push(ImportSkips {
            //     change_set_pk: ChangeSetPk::NONE,
            //     attribute_skips,
            //     edge_skips,
            // });

            // for change_set in change_sets {
            //     if change_set.name() == default_change_set_name {
            //         continue;
            //     }

            //     // Revert to head to create new change set
            //     let ctx = ctx.clone_with_new_visibility(ctx.visibility().to_head());
            //     let new_cs = ChangeSet::new(&ctx, change_set.name(), None).await?;
            //     // Switch to new change set visibility
            //     let ctx = ctx.clone_with_new_visibility(ctx.visibility().to_change_set(new_cs.pk));

            //     let (_, attribute_skips, edge_skips) = import_change_set(
            //         &ctx,
            //         Some(new_cs.pk),
            //         &metadata,
            //         &change_set.funcs()?,
            //         &change_set.schemas()?,
            //         &change_set.components()?,
            //         &change_set.edges()?,
            //         installed_pkg_id,
            //         &mut change_set_things,
            //         &options,
            //     )
            //     .await?;

            //     import_skips.push(ImportSkips {
            //         change_set_pk: new_cs.pk,
            //         attribute_skips,
            //         edge_skips,
            //     });
            // }

            Ok((None, vec![], None))
        }
    }
}

pub async fn import_pkg(ctx: &DalContext, pkg_file_path: impl AsRef<Path>) -> PkgResult<SiPkg> {
    println!("Importing package from {:?}", pkg_file_path.as_ref());
    let pkg = SiPkg::load_from_file(&pkg_file_path).await?;

    import_pkg_from_pkg(ctx, &pkg, None).await?;

    Ok(pkg)
}

async fn create_func(
    ctx: &DalContext,
    func_spec: &SiPkgFunc<'_>,
    is_builtin: bool,
) -> PkgResult<Func> {
    let name = func_spec.name();

    let func_spec_data = func_spec
        .data()
        .ok_or(PkgError::DataNotFound(name.into()))?;

    let func = Func::new(
        ctx,
        name,
        func_spec_data
            .display_name()
            .map(|display_name| display_name.to_owned()),
        func_spec_data.description().map(|desc| desc.to_owned()),
        func_spec_data.link().map(|l| l.to_string()),
        func_spec_data.hidden(),
        is_builtin,
        func_spec_data.backend_kind().into(),
        func_spec_data.response_type().into(),
        Some(func_spec_data.handler().to_owned()),
        Some(func_spec_data.code_base64().to_owned()),
    )
    .await?;

    Ok(func)
}

#[allow(dead_code)]
async fn update_func(
    ctx: &DalContext,
    func: Func,
    func_spec_data: &SiPkgFuncData,
) -> PkgResult<Func> {
    let func = func
        .modify(ctx, |func| {
            func.name = func_spec_data.name().to_owned();
            func.backend_kind = func_spec_data.backend_kind().into();
            func.backend_response_type = func_spec_data.response_type().into();
            func.display_name = func_spec_data
                .display_name()
                .map(|display_name| display_name.to_owned());
            func.code_base64 = Some(func_spec_data.code_base64().to_owned());
            func.description = func_spec_data.description().map(|desc| desc.to_owned());
            func.handler = Some(func_spec_data.handler().to_owned());
            func.hidden = func_spec_data.hidden();
            func.link = func_spec_data.link().map(|l| l.to_string());

            Ok(())
        })
        .await?;

    Ok(func)
}

pub async fn import_func(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    func_spec: &SiPkgFunc<'_>,
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
    is_builtin: bool,
) -> PkgResult<Option<Func>> {
    let func = match change_set_pk {
        None => {
            let hash = func_spec.hash().to_string();
            let existing_func =
                InstalledPkgAsset::list_for_kind_and_hash(ctx, InstalledPkgAssetKind::Func, &hash)
                    .await?
                    .pop();

            let (func, created) = match existing_func {
                Some(installed_func_record) => match installed_func_record.as_installed_func()? {
                    InstalledPkgAssetTyped::Func { id, .. } => {
                        (Func::get_by_id(ctx, id).await?, false)
                    }
                    _ => unimplemented!("no idea what happens here!"),
                },
                None => (create_func(ctx, func_spec, is_builtin).await?, true),
            };

            if let Some(installed_pkg_id) = installed_pkg_id {
                InstalledPkgAsset::new(
                    ctx,
                    InstalledPkgAssetTyped::new_for_func(func.id, installed_pkg_id, hash),
                )
                .await?;
            }

            thing_map.insert(
                change_set_pk,
                func_spec.unique_id().to_owned(),
                Thing::Func(func.to_owned()),
            );

            if created {
                Some(func)
            } else {
                None
            }
        }
        Some(_) => {
            unimplemented!("workspace import not fixed");
            // let existing_func = thing_map.get(change_set_pk, &func_spec.unique_id().to_owned());

            // match existing_func {
            //     Some(Thing::Func(existing_func)) => {
            //         let mut existing_func = existing_func.to_owned();

            //         if func_spec.deleted() {
            //             existing_func.delete_by_id(ctx).await?;

            //             None
            //         } else {
            //             if let Some(data) = func_spec.data() {
            //                 update_func(ctx, &mut existing_func, data).await?;
            //             }

            //             Some(existing_func)
            //         }
            //     }
            //     _ => {
            //         if func_spec.deleted() {
            //             // If we're "deleted" but there is no existing function, this means we're
            //             // deleted only in a change set. Do nothing
            //             None
            //         } else {
            //             Some(create_func(ctx, func_spec).await?)
            //         }
            //     }
            // }
        }
    };

    if let Some(func) = func.as_ref() {
        thing_map.insert(
            change_set_pk,
            func_spec.unique_id().to_owned(),
            Thing::Func(func.to_owned()),
        );
    }

    Ok(func)
}

async fn create_func_argument(
    ctx: &DalContext,
    func_id: FuncId,
    func_arg: &SiPkgFuncArgument<'_>,
) -> PkgResult<FuncArgument> {
    Ok(FuncArgument::new(
        ctx,
        func_arg.name(),
        func_arg.kind().into(),
        func_arg.element_kind().to_owned().map(|&kind| kind.into()),
        func_id,
    )
    .await?)
}

// async fn update_func_argument(
//     ctx: &DalContext,
//     existing_arg: &mut FuncArgument,
//     func_id: FuncId,
//     func_arg: &SiPkgFuncArgument<'_>,
// ) -> PkgResult<()> {
//     existing_arg.set_name(ctx, func_arg.name()).await?;
//     existing_arg.set_kind(ctx, func_arg.kind()).await?;
//     let element_kind: Option<FuncArgumentKind> = func_arg.element_kind().map(|&kind| kind.into());
//     existing_arg.set_element_kind(ctx, element_kind).await?;
//     existing_arg.set_func_id(ctx, func_id).await?;
//
//     Ok(())
// }

async fn import_func_arguments(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    func_id: FuncId,
    func_arguments: &[SiPkgFuncArgument<'_>],
    _thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match change_set_pk {
        None => {
            for arg in func_arguments {
                create_func_argument(ctx, func_id, arg).await?;
            }
        }
        Some(_) => {} //             for arg in func_arguments {
                      //                 let unique_id =
                      //                     arg.unique_id()
                      //                         .ok_or(PkgError::MissingUniqueIdForNode(format!(
                      //                             "func-argument-{}",
                      //                             arg.hash()
                      //                         )))?;
                      //
                      //                 match thing_map.get(change_set_pk, &unique_id.to_owned()) {
                      //                     Some(Thing::FuncArgument(existing_arg)) => {
                      //                         let mut existing_arg = existing_arg.to_owned();
                      //
                      //                         if arg.deleted() {
                      //                             existing_arg.delete_by_id(ctx).await?;
                      //                         } else {
                      //                             update_func_argument(ctx, &mut existing_arg, func_id, arg).await?;
                      //                             thing_map.insert(
                      //                                 change_set_pk,
                      //                                 unique_id.to_owned(),
                      //                                 Thing::FuncArgument(existing_arg.to_owned()),
                      //                             );
                      //                         }
                      //                     }
                      //                     _ => {
                      //                         if !arg.deleted() {
                      //                             let new_arg = create_func_argument(ctx, func_id, arg).await?;
                      //                             thing_map.insert(
                      //                                 change_set_pk,
                      //                                 unique_id.to_owned(),
                      //                                 Thing::FuncArgument(new_arg),
                      //                             );
                      //                         }
                      //                     }
                      //                 }
                      //             }
                      //         }
    }

    Ok(())
}

async fn create_schema(ctx: &DalContext, schema_spec_data: &SiPkgSchemaData) -> PkgResult<Schema> {
    let schema = Schema::new(ctx, schema_spec_data.name(), ComponentKind::Standard)
        .await?
        .modify(ctx, |schema| {
            schema.ui_hidden = schema_spec_data.ui_hidden();
            Ok(())
        })
        .await?;
    Ok(schema)
}

// async fn update_schema(
//     ctx: &DalContext,
//     schema: &mut Schema,
//     schema_spec_data: &SiPkgSchemaData,
// ) -> PkgResult<()> {
//     if schema_spec_data.name() != schema.name() {
//         schema.set_name(ctx, schema_spec_data.name()).await?;
//     }

//     if schema_spec_data.ui_hidden() != schema.ui_hidden() {
//         schema
//             .set_ui_hidden(ctx, schema_spec_data.ui_hidden())
//             .await?;
//     }

//     if let Some(mut ui_menu) = schema.ui_menus(ctx).await?.pop() {
//         if let Some(category_name) = schema_spec_data.category_name() {
//             if category_name != ui_menu.name() {
//                 ui_menu.set_name(ctx, category_name).await?;
//             }
//             if schema_spec_data.category() != ui_menu.category() {
//                 ui_menu.set_name(ctx, schema_spec_data.category()).await?;
//             }
//         }
//     }

//     Ok(())
// }

async fn import_schema(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    schema_spec: &SiPkgSchema<'_>,
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
) -> PkgResult<(Option<SchemaId>, Vec<SchemaVariantId>)> {
    let schema_and_category = match change_set_pk {
        None => {
            let hash = schema_spec.hash().to_string();
            let existing_schema = InstalledPkgAsset::list_for_kind_and_hash(
                ctx,
                InstalledPkgAssetKind::Schema,
                &hash,
            )
            .await?
            .pop();

            let data = schema_spec
                .data()
                .ok_or(PkgError::DataNotFound("schema".into()))?;

            // NOTE(nick): with the new engine, the category moves to the schema variant, so we need
            // to pull it off here, even if we find an existing schema.
            let category = data.category.clone();

            let schema = match existing_schema {
                None => create_schema(ctx, data).await?,
                Some(installed_schema_record) => {
                    match installed_schema_record.as_installed_schema()? {
                        InstalledPkgAssetTyped::Schema { id, .. } => {
                            Schema::get_by_id(ctx, id).await?
                        }
                        _ => unimplemented!("no idea what happens here!"),
                    }
                }
            };

            // Even if the asset is already installed, we write a record of the asset installation so that
            // we can track the installed packages that share schemas.
            if let Some(installed_pkg_id) = installed_pkg_id {
                InstalledPkgAsset::new(
                    ctx,
                    InstalledPkgAssetTyped::new_for_schema(schema.id(), installed_pkg_id, hash),
                )
                .await?;
            }

            Some((schema, category))
        }
        Some(_) => {
            unimplemented!("workspace import not yet implemented")
            // let unique_id = schema_spec
            //     .unique_id()
            //     .ok_or(PkgError::MissingUniqueIdForNode(format!(
            //         "schema {}",
            //         schema_spec.hash()
            //     )))?;
            //
            // match thing_map.get(change_set_pk, &unique_id.to_owned()) {
            //     Some(Thing::Schema(schema)) => {
            //         let mut schema = schema.to_owned();
            //
            //         if schema_spec.deleted() {
            //             schema.delete_by_id(ctx).await?;
            //             // delete all schema children?
            //
            //             None
            //         } else {
            //             if let Some(data) = schema_spec.data() {
            //                 update_schema(ctx, &mut schema, data).await?;
            //             }
            //
            //             Some(schema)
            //         }
            //     }
            //     _ => {
            //         if schema_spec.deleted() {
            //             None
            //         } else {
            //             Some(
            //                 create_schema(
            //                     ctx,
            //                     schema_spec
            //                         .data()
            //                         .ok_or(PkgError::DataNotFound("schema".into()))?,
            //                 )
            //                 .await?,
            //             )
            //         }
            //     }
            // }
        }
    };

    if let Some((mut schema, category)) = schema_and_category {
        if let Some(unique_id) = schema_spec.unique_id() {
            thing_map.insert(
                change_set_pk,
                unique_id.to_owned(),
                Thing::Schema(schema.to_owned()),
            );
        }

        let installed_schema_variant_ids = vec![];
        for variant_spec in &schema_spec.variants()? {
            let _variant = import_schema_variant(
                ctx,
                change_set_pk,
                &mut schema,
                category.clone(),
                variant_spec,
                installed_pkg_id,
                thing_map,
            )
            .await?;

            // if let Some(variant) = variant {
            //     installed_schema_variant_ids.push(*variant.id());
            //
            //     if let Some(variant_spec_data) = variant_spec.data() {
            //         let func_unique_id = variant_spec_data.func_unique_id().to_owned();
            //
            //         set_default_schema_variant_id(
            //             ctx,
            //             change_set_pk,
            //             &mut schema,
            //             schema_spec
            //                 .data()
            //                 .as_ref()
            //                 .and_then(|data| data.default_schema_variant()),
            //             variant_spec.unique_id(),
            //             *variant.id(),
            //         )
            //         .await?;
            //
            //         if let Thing::Func(asset_func) =
            //             thing_map
            //                 .get(change_set_pk, &func_unique_id)
            //                 .ok_or(PkgError::MissingFuncUniqueId(func_unique_id.to_string()))?
            //         {
            //             create_schema_variant_definition(
            //                 ctx,
            //                 schema_spec.clone(),
            //                 installed_pkg_id,
            //                 *variant.id(),
            //                 asset_func,
            //             )
            //             .await?;
            //         }
            //     }
            // }
        }

        Ok((Some(schema.id()), installed_schema_variant_ids))
    } else {
        Ok((None, vec![]))
    }
}

// async fn set_default_schema_variant_id(
//     ctx: &DalContext,
//     change_set_pk: Option<ChangeSetPk>,
//     schema: &mut Schema,
//     spec_default_unique_id: Option<&str>,
//     variant_unique_id: Option<&str>,
//     variant_id: SchemaVariantId,
// ) -> PkgResult<()> {
//     match (change_set_pk, variant_unique_id, spec_default_unique_id) {
//         (None, _, _) | (Some(_), None, _) | (_, Some(_), None) => {
//             if schema.default_schema_variant_id().is_none() {
//                 schema
//                     .set_default_schema_variant_id(ctx, Some(variant_id))
//                     .await?;
//             }
//         }
//         (Some(_), Some(variant_unique_id), Some(spec_default_unique_id)) => {
//             if variant_unique_id == spec_default_unique_id {
//                 let current_default_variant_id = schema
//                     .default_schema_variant_id()
//                     .copied()
//                     .unwrap_or(SchemaVariantId::NONE);

//                 if variant_id != current_default_variant_id {
//                     schema
//                         .set_default_schema_variant_id(ctx, Some(variant_id))
//                         .await?;
//                 }
//             }
//         }
//     }

//     Ok(())
// }

// async fn create_schema_variant_definition(
//     ctx: &DalContext,
//     schema_spec: SiPkgSchema<'_>,
//     installed_pkg_id: Option<InstalledPkgId>,
//     schema_variant_id: SchemaVariantId,
//     asset_func: &Func,
// ) -> PkgResult<()> {
//     let hash = schema_spec.hash().to_string();
//     let existing_definition = InstalledPkgAsset::list_for_kind_and_hash(
//         ctx,
//         InstalledPkgAssetKind::SchemaVariantDefinition,
//         &hash,
//     )
//     .await?
//     .pop();

//     let definition = match existing_definition {
//         None => {
//             let maybe_schema_variant_definition =
//                 SchemaVariantDefinition::get_by_func_id(ctx, *asset_func.id()).await?;
//             let mut schema_variant_definition = match maybe_schema_variant_definition {
//                 None => {
//                     let spec = schema_spec.to_spec().await?;
//                     let metadata = SchemaVariantDefinitionJson::metadata_from_spec(spec)?;

//         let mut svd = SchemaVariantDefinition::new(
//             ctx,
//             metadata.name,
//             metadata.menu_name,
//             metadata.category,
//             metadata.link,
//             metadata.color,
//             metadata.component_kind,
//             metadata.description,
//             *asset_func.id(),
//         )
//         .await?;

//         svd.set_component_type(ctx, metadata.component_type).await?;
//         svd
//     }
//     Some(schema_variant_definition) => schema_variant_definition,
// };

//             schema_variant_definition
//                 .set_schema_variant_id(ctx, Some(schema_variant_id))
//                 .await?;

//             schema_variant_definition
//         }
//         Some(existing_definition) => {
//             match existing_definition.as_installed_schema_variant_definition()? {
//                 InstalledPkgAssetTyped::SchemaVariantDefinition { id, .. } => {
//                     match SchemaVariantDefinition::get_by_id(ctx, &id).await? {
//                         Some(definition) => definition,
//                         None => return Err(PkgError::InstalledSchemaVariantDefinitionMissing(id)),
//                     }
//                 }
//                 _ => unreachable!(
//                     "we are protected by the as_installed_schema_variant_definition method"
//                 ),
//             }
//         }
//     };

//     if let Some(installed_pkg_id) = installed_pkg_id {
//         InstalledPkgAsset::new(
//             ctx,
//             InstalledPkgAssetTyped::new_for_schema_variant_definition(
//                 *definition.id(),
//                 installed_pkg_id,
//                 hash,
//             ),
//         )
//         .await?;
//     }

//     Ok(())
// }

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct AttrFuncInfo {
    func_unique_id: String,
    prop_id: PropId,
    inputs: Vec<SiPkgAttrFuncInputView>,
}

#[allow(dead_code)]
#[remain::sorted]
#[derive(Clone, Debug)]
enum DefaultValueInfo {
    Boolean {
        prop_id: PropId,
        default_value: bool,
    },
    Number {
        prop_id: PropId,
        default_value: i64,
    },
    String {
        prop_id: PropId,
        default_value: String,
    },
}

struct PropVisitContext<'a> {
    pub ctx: &'a DalContext,
    pub schema_variant_id: SchemaVariantId,
    pub attr_funcs: Mutex<Vec<AttrFuncInfo>>,
    pub default_values: Mutex<Vec<DefaultValueInfo>>,
    pub map_key_funcs: Mutex<Vec<(String, AttrFuncInfo)>>,
    pub change_set_pk: Option<ChangeSetPk>,
}

async fn import_leaf_function(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    leaf_func: SiPkgLeafFunction<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let inputs: Vec<LeafInputLocation> = leaf_func
        .inputs()
        .iter()
        .map(|input| input.into())
        .collect();

    let kind: LeafKind = leaf_func.leaf_kind().into();

    match thing_map.get(change_set_pk, &leaf_func.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            SchemaVariant::upsert_leaf_function(ctx, schema_variant_id, None, kind, &inputs, func)
                .await?;
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                leaf_func.func_unique_id().to_string(),
            ));
        }
    }

    Ok(())
}

async fn get_identity_func(ctx: &DalContext) -> PkgResult<FuncId> {
    Ok(Func::find_intrinsic(ctx, IntrinsicFunc::Identity).await?)
}

async fn create_socket(
    ctx: &DalContext,
    data: &SiPkgSocketData,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<(Option<InputSocket>, Option<OutputSocket>)> {
    let identity_func_id = get_identity_func(ctx).await?;

    // Connection annotations are stored as a serialized json array of strings
    let connection_annotations: Vec<String> = serde_json::from_str(data.connection_annotations())?;

    let (ip, ep) = match data.kind() {
        SocketSpecKind::Input => {
            let ip = InputSocket::new(
                ctx,
                schema_variant_id,
                data.name(),
                identity_func_id,
                data.arity().into(),
                SocketKind::Standard,
                connection_annotations,
            )
            .await?;

            (Some(ip), None)
        }
        SocketSpecKind::Output => {
            let ep = OutputSocket::new(
                ctx,
                schema_variant_id,
                data.name(),
                None,
                identity_func_id,
                data.arity().into(),
                SocketKind::Standard,
                connection_annotations,
            )
            .await?;

            (None, Some(ep))
        }
    };

    // TODO: add modify_by_id to socket, ui hide frames
    // socket.set_ui_hidden(ctx, data.ui_hidden()).await?;

    Ok((ip, ep))
}

async fn import_socket(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    socket_spec: SiPkgSocket<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let (ip, ep) = match change_set_pk {
        None => {
            let data = socket_spec
                .data()
                .ok_or(PkgError::DataNotFound(socket_spec.name().into()))?;

            create_socket(ctx, data, schema_variant_id).await?
        }
        Some(_) => {
            todo!("workspace backup imports");
            //            let unique_id = socket_spec
            //                .unique_id()
            //                .ok_or(PkgError::MissingUniqueIdForNode(format!(
            //                    "socket {}",
            //                    socket_spec.hash()
            //                )))?;
            //
            //            match thing_map.get(change_set_pk, &unique_id.to_owned()) {
            //                Some(Thing::Socket(socket_box)) => {
            //                    (
            //                        socket_box.0.to_owned(),
            //                        socket_box.1.to_owned(),
            //                        socket_box.2.to_owned(),
            //                    )
            //                    // prop trees, including sockets and providers, are created whole cloth, so
            //                    // should not have differences in change sets (currently)
            //                }
            //                _ => {
            //                    let data = socket_spec
            //                        .data()
            //                        .ok_or(PkgError::DataNotFound(socket_spec.name().into()))?;
            //
            //                    create_socket(ctx, data, schema_id, schema_variant_id).await?
            //                }
            //            }
        }
    };

    if let Some(unique_id) = socket_spec.unique_id() {
        thing_map.insert(
            change_set_pk,
            unique_id.to_owned(),
            Thing::Socket(Box::new((ip.to_owned(), ep.to_owned()))),
        );
    }

    match (
        socket_spec.data().and_then(|data| data.func_unique_id()),
        ep,
        ip,
    ) {
        (Some(func_unique_id), Some(ep), None) => {
            import_attr_func_for_output_socket(
                ctx,
                change_set_pk,
                schema_variant_id,
                ep.id(),
                func_unique_id,
                socket_spec.inputs()?.drain(..).map(Into::into).collect(),
                thing_map,
            )
            .await?;
        }
        (Some(_), _, Some(_)) => {}
        _ => {}
    }

    Ok(())
}

async fn create_action_protoype(
    ctx: &DalContext,
    action_func_spec: &SiPkgActionFunc<'_>,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<ActionPrototype> {
    let proto = ActionPrototype::new(
        ctx,
        action_func_spec.name(),
        action_func_spec.kind().into(),
        schema_variant_id,
        func_id,
    )
    .await?;

    Ok(proto)
}

// async fn update_action_prototype(
//     ctx: &DalContext,
//     prototype: &mut ActionPrototype,
//     action_func_spec: &SiPkgActionFunc<'_>,
//     func_id: FuncId,
//     schema_variant_id: SchemaVariantId,
// ) -> PkgResult<()> {
//     if prototype.schema_variant_id() != schema_variant_id {
//         prototype
//             .set_schema_variant_id(ctx, schema_variant_id)
//             .await?;
//     }

//     if prototype.name() != action_func_spec.name() {
//         prototype.set_name(ctx, action_func_spec.name()).await?;
//     }

//     if prototype.func_id() != func_id {
//         prototype.set_func_id(ctx, func_id).await?;
//     }

//     let kind: ActionKind = action_func_spec.kind().into();
//     if *prototype.kind() != kind {
//         prototype.set_kind(ctx, kind).await?;
//     }

//     Ok(())
// }

async fn import_action_func(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    action_func_spec: &SiPkgActionFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<ActionPrototype>> {
    let prototype =
        match thing_map.get(change_set_pk, &action_func_spec.func_unique_id().to_owned()) {
            Some(Thing::Func(func)) => {
                let func_id = func.id;

                if let Some(unique_id) = action_func_spec.unique_id() {
                    match thing_map.get(change_set_pk, &unique_id.to_owned()) {
                        Some(Thing::ActionPrototype(_prototype)) => {
                            todo!("workspace import paths not yet implemented");
                            //                            let mut prototype = prototype.to_owned();
                            //
                            //                            if action_func_spec.deleted() {
                            //                                prototype.delete_by_id(ctx).await?;
                            //                            } else {
                            //                                update_action_prototype(
                            //                                    ctx,
                            //                                    &mut prototype,
                            //                                    action_func_spec,
                            //                                    func_id,
                            //                                    schema_variant_id,
                            //                                )
                            //                                .await?;
                            //                            }
                            //
                            //                            Some(prototype)
                        }
                        _ => {
                            if action_func_spec.deleted() {
                                None
                            } else {
                                Some(
                                    create_action_protoype(
                                        ctx,
                                        action_func_spec,
                                        func_id,
                                        schema_variant_id,
                                    )
                                    .await?,
                                )
                            }
                        }
                    }
                } else {
                    Some(
                        create_action_protoype(ctx, action_func_spec, func_id, schema_variant_id)
                            .await?,
                    )
                }
            }
            _ => {
                return Err(PkgError::MissingFuncUniqueId(
                    action_func_spec.func_unique_id().into(),
                ));
            }
        };

    Ok(prototype)
}

async fn import_auth_func(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    func_spec: &SiPkgAuthFunc<'_>,
    schema_variant_id: SchemaVariantId,
    thing_map: &ThingMap,
) -> PkgResult<Option<AuthenticationPrototype>> {
    let prototype = match thing_map.get(change_set_pk, &func_spec.func_unique_id().to_owned()) {
        Some(Thing::Func(func)) => {
            let func_id = func.id;

            if let Some(unique_id) = func_spec.unique_id() {
                match thing_map.get(change_set_pk, &unique_id.to_owned()) {
                    Some(Thing::AuthPrototype(_prototype)) => {
                        todo!("workspace import paths not yet implemented");
                        // AuthenticationPrototype is represented by just and edge,
                        // Since the info that matters is only then func_id and the schema_variant_id
                        // Do we need to update it?

                        // let mut prototype = prototype.to_owned();
                        //
                        // if func_spec.deleted() {
                        //     prototype.delete_by_id(ctx).await?;
                        // } else {
                        //     update_authentication_prototype(
                        //         ctx,
                        //         &mut prototype,
                        //         func_id,
                        //         schema_variant_id,
                        //     )
                        //     .await?;
                        // }
                        //
                        // Some(prototype)
                    }
                    _ => {
                        if func_spec.deleted() {
                            None
                        } else {
                            SchemaVariant::new_authentication_prototype(
                                ctx,
                                func_id,
                                schema_variant_id,
                            )
                            .await?;

                            Some(AuthenticationPrototype {
                                id: AuthenticationPrototypeId::generate(),
                                func_id,
                                schema_variant_id,
                            })
                        }
                    }
                }
            } else {
                SchemaVariant::new_authentication_prototype(ctx, func_id, schema_variant_id)
                    .await?;
                Some(AuthenticationPrototype {
                    id: AuthenticationPrototypeId::generate(),
                    func_id,
                    schema_variant_id,
                })
            }
        }
        _ => {
            return Err(PkgError::MissingFuncUniqueId(
                func_spec.func_unique_id().into(),
            ));
        }
    };

    Ok(prototype)
}

#[derive(Default, Clone, Debug)]
struct CreatePropsSideEffects {
    attr_funcs: Vec<AttrFuncInfo>,
    default_values: Vec<DefaultValueInfo>,
    map_key_funcs: Vec<(String, AttrFuncInfo)>,
}

impl IntoIterator for CreatePropsSideEffects {
    type Item = CreatePropsSideEffects;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

impl Extend<CreatePropsSideEffects> for CreatePropsSideEffects {
    fn extend<T: IntoIterator<Item = CreatePropsSideEffects>>(&mut self, iter: T) {
        for element in iter {
            self.attr_funcs.extend(element.attr_funcs);
            self.default_values.extend(element.default_values);
            self.map_key_funcs.extend(element.map_key_funcs);
        }
    }
}

async fn create_props(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    variant_spec: &SiPkgSchemaVariant<'_>,
    prop_root: SchemaVariantSpecPropRoot,
    prop_root_prop_id: PropId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<CreatePropsSideEffects> {
    let context = PropVisitContext {
        ctx,
        schema_variant_id,
        attr_funcs: Mutex::new(vec![]),
        default_values: Mutex::new(vec![]),
        map_key_funcs: Mutex::new(vec![]),
        change_set_pk,
    };

    let parent_info = ParentPropInfo {
        prop_id: prop_root_prop_id,
        path: PropPath::new(prop_root.path_parts()),
        kind: PropKind::Object,
    };

    variant_spec
        .visit_prop_tree(prop_root, create_prop, Some(parent_info), &context)
        .await?;

    Ok(CreatePropsSideEffects {
        attr_funcs: context.attr_funcs.into_inner(),
        default_values: context.default_values.into_inner(),
        map_key_funcs: context.map_key_funcs.into_inner(),
    })
}

// async fn update_schema_variant(
//     ctx: &DalContext,
//     schema_variant: &mut SchemaVariant,
//     name: &str,
//     schema_id: SchemaId,
// ) -> PkgResult<()> {
//     let current_schema_id = schema_variant
//         .schema(ctx)
//         .await?
//         .map(|schema| *schema.id())
//         .ok_or(SchemaVariantError::MissingSchema(*schema_variant.id()))?;

//     if schema_id != current_schema_id {
//         schema_variant.set_schema(ctx, &schema_id).await?;
//     }

//     if schema_variant.name() != name {
//         schema_variant.set_name(ctx, name).await?;
//     }

//     Ok(())
// }

async fn import_schema_variant(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    schema: &mut Schema,
    category: String,
    variant_spec: &SiPkgSchemaVariant<'_>,
    installed_pkg_id: Option<InstalledPkgId>,
    thing_map: &mut ThingMap,
) -> PkgResult<Option<SchemaVariant>> {
    let schema_variant = match change_set_pk {
        None => {
            let hash = variant_spec.hash().to_string();
            let existing_schema_variant = InstalledPkgAsset::list_for_kind_and_hash(
                ctx,
                InstalledPkgAssetKind::SchemaVariant,
                &hash,
            )
            .await?
            .pop();

            let (variant, created) = match existing_schema_variant {
                Some(installed_sv_record) => {
                    match installed_sv_record.as_installed_schema_variant()? {
                        InstalledPkgAssetTyped::SchemaVariant { id, .. } => (SchemaVariant::get_by_id(ctx, id).await?, false),
                        _ => unreachable!(
                            "the as_installed_schema_variant method ensures we cannot hit this branch"
                        ),
                    }
                }
                None => (
                    // FIXME(nick): move category, color, and all metadata to variant or somewhere
                    // else. It should not be on schema.
                    SchemaVariant::new(ctx, schema.id(), variant_spec.name(), category)
                        .await?
                        .0,
                    true,
                ),
            };

            if let Some(installed_pkg_id) = installed_pkg_id {
                InstalledPkgAsset::new(
                    ctx,
                    InstalledPkgAssetTyped::new_for_schema_variant(
                        variant.id(),
                        installed_pkg_id,
                        hash,
                    ),
                )
                .await?;
            }

            if created {
                Some(variant)
            } else {
                None
            }
        }
        Some(_) => {
            unimplemented!("workspace import is not working at this time")
            // let unique_id = variant_spec
            //     .unique_id()
            //     .ok_or(PkgError::MissingUniqueIdForNode(format!(
            //         "variant {}",
            //         variant_spec.hash()
            //     )))?;
            //
            // match thing_map.get(change_set_pk, &unique_id.to_owned()) {
            //     Some(Thing::SchemaVariant(variant)) => {
            //         let mut variant = variant.to_owned();
            //         update_schema_variant(ctx, &mut variant, variant_spec.name(), *schema.id())
            //             .await?;
            //
            //         if variant_spec.deleted() {
            //             variant.delete_by_id(ctx).await?;
            //
            //             None
            //         } else {
            //             Some(variant)
            //         }
            //     }
            //     _ => {
            //         if variant_spec.deleted() {
            //             None
            //         } else {
            //             Some(
            //                 SchemaVariant::new(ctx, *schema.id(), variant_spec.name())
            //                     .await?
            //                     .0,
            //             )
            //         }
            //     }
            // }
        }
    };

    let schema_variant = match schema_variant {
        None => None,
        Some(schema_variant) => {
            if let Some(unique_id) = variant_spec.unique_id() {
                thing_map.insert(
                    change_set_pk,
                    unique_id.to_owned(),
                    Thing::SchemaVariant(schema_variant.to_owned()),
                );
            }

            if let Some(data) = variant_spec.data() {
                if let Some(color) = data.color() {
                    let current_color = schema_variant.get_color(ctx).await?;
                    if current_color.as_deref() != Some(color) {
                        schema_variant.set_color(ctx, color).await?
                    }
                }

                schema_variant
                    .set_type(ctx, data.component_type().to_string())
                    .await?;
            }

            let mut side_effects = CreatePropsSideEffects::default();

            let domain_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant.id(),
                &PropPath::new(["root", "domain"]),
            )
            .await?;

            side_effects.extend(
                create_props(
                    ctx,
                    change_set_pk,
                    variant_spec,
                    SchemaVariantSpecPropRoot::Domain,
                    domain_prop_id,
                    schema_variant.id(),
                )
                .await?,
            );

            let resource_value_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant.id(),
                &PropPath::new(["root", "resource_value"]),
            )
            .await?;

            side_effects.extend(
                create_props(
                    ctx,
                    change_set_pk,
                    variant_spec,
                    SchemaVariantSpecPropRoot::ResourceValue,
                    resource_value_prop_id,
                    schema_variant.id(),
                )
                .await?,
            );

            let secrets_prop_id = Prop::find_prop_id_by_path(
                ctx,
                schema_variant.id(),
                &PropPath::new(["root", "secrets"]),
            )
            .await?;

            side_effects.extend(
                create_props(
                    ctx,
                    change_set_pk,
                    variant_spec,
                    SchemaVariantSpecPropRoot::Secrets,
                    secrets_prop_id,
                    schema_variant.id(),
                )
                .await?,
            );

            if !variant_spec.secret_definitions()?.is_empty() {
                let root_prop_id =
                    Prop::find_prop_id_by_path(ctx, schema_variant.id(), &PropPath::new(["root"]))
                        .await?;

                let secret_definition_prop = Prop::new(
                    ctx,
                    "secret_definition",
                    PropKind::Object,
                    false,
                    None,
                    None,
                    PropParent::OrderedProp(root_prop_id),
                )
                .await?;
                let secret_definition_prop_id = secret_definition_prop.id();

                side_effects.extend(
                    create_props(
                        ctx,
                        change_set_pk,
                        variant_spec,
                        SchemaVariantSpecPropRoot::SecretDefinition,
                        secret_definition_prop_id,
                        schema_variant.id(),
                    )
                    .await?,
                );
            }

            SchemaVariant::finalize(ctx, schema_variant.id()).await?;

            for socket in variant_spec.sockets()? {
                import_socket(ctx, change_set_pk, socket, schema_variant.id(), thing_map).await?;
            }

            for action_func in &variant_spec.action_funcs()? {
                let prototype = import_action_func(
                    ctx,
                    change_set_pk,
                    action_func,
                    schema_variant.id(),
                    thing_map,
                )
                .await?;

                if let (Some(prototype), Some(unique_id)) = (prototype, action_func.unique_id()) {
                    thing_map.insert(
                        change_set_pk,
                        unique_id.to_owned(),
                        Thing::ActionPrototype(prototype),
                    );
                }
            }

            for auth_func in &variant_spec.auth_funcs()? {
                let prototype = import_auth_func(
                    ctx,
                    change_set_pk,
                    auth_func,
                    schema_variant.id(),
                    thing_map,
                )
                .await?;

                if let (Some(prototype), Some(unique_id)) = (prototype, auth_func.unique_id()) {
                    thing_map.insert(
                        change_set_pk,
                        unique_id.to_owned(),
                        Thing::AuthPrototype(prototype),
                    );
                }
            }

            for leaf_func in variant_spec.leaf_functions()? {
                import_leaf_function(
                    ctx,
                    change_set_pk,
                    leaf_func,
                    schema_variant.id(),
                    thing_map,
                )
                .await?;
            }

            // Default values must be set before attribute functions are configured so they don't
            // override the prototypes set there
            for default_value_info in side_effects.default_values {
                set_default_value(ctx, default_value_info).await?;
            }

            // Set a default name value for all name props, this ensures region has a name before
            // the function is executed
            {
                let name_prop_id = Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant.id(),
                    &PropPath::new(["root", "si", "name"]),
                )
                .await?;
                let name_default_value_info = DefaultValueInfo::String {
                    prop_id: name_prop_id,
                    default_value: schema.name.to_owned().to_lowercase(),
                };

                set_default_value(ctx, name_default_value_info).await?;
            }

            for si_prop_func in variant_spec.si_prop_funcs()? {
                let prop_id = Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant.id(),
                    &PropPath::new(si_prop_func.kind().prop_path()),
                )
                .await?;
                import_attr_func_for_prop(
                    ctx,
                    change_set_pk,
                    schema_variant.id(),
                    AttrFuncInfo {
                        func_unique_id: si_prop_func.func_unique_id().to_owned(),
                        prop_id,
                        inputs: si_prop_func
                            .inputs()?
                            .iter()
                            .map(|input| input.to_owned().into())
                            .collect(),
                    },
                    None,
                    thing_map,
                )
                .await?;
            }

            let mut has_resource_value_func = false;
            for root_prop_func in variant_spec.root_prop_funcs()? {
                if root_prop_func.prop() == SchemaVariantSpecPropRoot::ResourceValue {
                    has_resource_value_func = true;
                }

                let prop_id = Prop::find_prop_id_by_path(
                    ctx,
                    schema_variant.id(),
                    &PropPath::new(root_prop_func.prop().path_parts()),
                )
                .await?;
                import_attr_func_for_prop(
                    ctx,
                    change_set_pk,
                    schema_variant.id(),
                    AttrFuncInfo {
                        func_unique_id: root_prop_func.func_unique_id().to_owned(),
                        prop_id,
                        inputs: root_prop_func
                            .inputs()?
                            .iter()
                            .map(|input| input.to_owned().into())
                            .collect(),
                    },
                    None,
                    thing_map,
                )
                .await?;
            }
            if !has_resource_value_func {
                attach_resource_payload_to_value(ctx, schema_variant.id()).await?;
            }

            for attr_func in side_effects.attr_funcs {
                import_attr_func_for_prop(
                    ctx,
                    change_set_pk,
                    schema_variant.id(),
                    attr_func,
                    None,
                    thing_map,
                )
                .await?;
            }

            for (key, map_key_func) in side_effects.map_key_funcs {
                import_attr_func_for_prop(
                    ctx,
                    change_set_pk,
                    schema_variant.id(),
                    map_key_func,
                    Some(key),
                    thing_map,
                )
                .await?;
            }

            Some(schema_variant)
        }
    };

    Ok(schema_variant)
}

async fn set_default_value(
    ctx: &DalContext,
    default_value_info: DefaultValueInfo,
) -> PkgResult<()> {
    let prop_id = match &default_value_info {
        DefaultValueInfo::Number { prop_id, .. }
        | DefaultValueInfo::String { prop_id, .. }
        | DefaultValueInfo::Boolean { prop_id, .. } => *prop_id,
    };

    match default_value_info {
        DefaultValueInfo::Boolean { default_value, .. } => {
            Prop::set_default_value(ctx, prop_id, default_value).await?
        }
        DefaultValueInfo::Number { default_value, .. } => {
            Prop::set_default_value(ctx, prop_id, default_value).await?
        }
        DefaultValueInfo::String { default_value, .. } => {
            Prop::set_default_value(ctx, prop_id, default_value).await?
        }
    }

    Ok(())
}

async fn import_attr_func_for_prop(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    schema_variant_id: SchemaVariantId,
    AttrFuncInfo {
        func_unique_id,
        prop_id,
        inputs,
    }: AttrFuncInfo,
    key: Option<String>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(change_set_pk, &func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                change_set_pk,
                AttrFuncContext::Prop(prop_id),
                key,
                schema_variant_id,
                func.id,
                inputs,
                thing_map,
            )
            .await?;
        }
        _ => return Err(PkgError::MissingFuncUniqueId(func_unique_id.to_string())),
    }

    Ok(())
}

async fn import_attr_func_for_output_socket(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    schema_variant_id: SchemaVariantId,
    output_socket_id: OutputSocketId,
    func_unique_id: &str,
    inputs: Vec<SiPkgAttrFuncInputView>,
    thing_map: &mut ThingMap,
) -> PkgResult<()> {
    match thing_map.get(change_set_pk, &func_unique_id.to_owned()) {
        Some(Thing::Func(func)) => {
            import_attr_func(
                ctx,
                change_set_pk,
                AttrFuncContext::OutputSocket(output_socket_id),
                None,
                schema_variant_id,
                func.id,
                inputs,
                thing_map,
            )
            .await?;
        }
        _ => return Err(PkgError::MissingFuncUniqueId(func_unique_id.to_string())),
    }

    Ok(())
}

async fn get_prototype_for_context(
    ctx: &DalContext,
    context: AttrFuncContext,
    key: Option<String>,
) -> PkgResult<AttributePrototypeId> {
    if key.is_some() {
        #[allow(clippy::infallible_destructuring_match)]
        let map_prop_id = match context {
            AttrFuncContext::Prop(prop_id) => prop_id,
            _ => Err(PkgError::AttributeFuncForKeyMissingProp(
                context,
                key.to_owned().expect("check above ensures this is some"),
            ))?,
        };
        let map_prop = Prop::get_by_id(ctx, map_prop_id).await?;

        if map_prop.kind != PropKind::Map {
            return Err(PkgError::AttributeFuncForKeySetOnWrongKind(
                map_prop_id,
                key.to_owned().expect("check above ensures this is some"),
                map_prop.kind,
            ));
        }

        let element_prop_id = map_prop.element_prop_id(ctx).await?;
        Ok(
            match AttributePrototype::find_for_prop(ctx, element_prop_id, &key).await? {
                None => {
                    let unset_func_id = Func::find_intrinsic(ctx, IntrinsicFunc::Unset).await?;
                    let prototype_id = AttributePrototype::new(ctx, unset_func_id).await?.id();
                    Prop::set_prototype_id(ctx, element_prop_id, prototype_id).await?;

                    prototype_id
                }
                Some(prototype_id) => prototype_id,
            },
        )
    } else {
        Ok(match context {
            AttrFuncContext::Prop(prop_id) => {
                AttributePrototype::find_for_prop(ctx, prop_id, &None)
                    .await?
                    .ok_or(PkgError::PropMissingPrototype(prop_id))?
            }
            AttrFuncContext::OutputSocket(output_socket_id) => {
                AttributePrototype::find_for_output_socket(ctx, output_socket_id)
                    .await?
                    .ok_or(PkgError::OutputSocketMissingPrototype(output_socket_id))?
            }
        })
    }
}

async fn create_attr_proto_arg(
    ctx: &DalContext,
    prototype_id: AttributePrototypeId,
    input: &SiPkgAttrFuncInputView,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<AttributePrototypeArgumentId> {
    let arg = match &input {
        SiPkgAttrFuncInputView::Prop { name, .. }
        | SiPkgAttrFuncInputView::InputSocket { name, .. }
        | SiPkgAttrFuncInputView::OutputSocket { name, .. } => {
            FuncArgument::find_by_name_for_func(ctx, name, func_id)
                .await?
                .ok_or(PkgError::MissingFuncArgument(name.to_owned(), func_id))?
        }
    };

    Ok(match input {
        SiPkgAttrFuncInputView::Prop { prop_path, .. } => {
            let prop_id =
                Prop::find_prop_id_by_path(ctx, schema_variant_id, &prop_path.into()).await?;
            let apa = AttributePrototypeArgument::new(ctx, prototype_id, arg.id).await?;
            let apa_id = apa.id();

            apa.set_value_from_prop_id(ctx, prop_id).await?;

            apa_id
        }
        SiPkgAttrFuncInputView::InputSocket { socket_name, .. } => {
            let input_socket = InputSocket::find_with_name(ctx, socket_name, schema_variant_id)
                .await?
                .ok_or(PkgError::MissingInputSocketName(socket_name.to_owned()))?;
            let apa = AttributePrototypeArgument::new(ctx, prototype_id, arg.id).await?;
            let apa_id = apa.id();

            apa.set_value_from_input_socket_id(ctx, input_socket.id())
                .await?;
            apa_id
        }
        _ => {
            // xxx: make this an error
            panic!("unsupported taking output socket as input for prop");
        }
    })
}

// async fn update_attr_proto_arg(
//     ctx: &DalContext,
//     apa: &mut AttributePrototypeArgument,
//     _prototype_id: AttributePrototypeId,
//     input: &SiPkgAttrFuncInputView,
//     func_id: FuncId,
//     schema_variant_id: SchemaVariantId,
// ) -> PkgResult<()> {
//     let arg = match &input {
//         SiPkgAttrFuncInputView::Prop { name, .. }
//         | SiPkgAttrFuncInputView::InputSocket { name, .. }
//         | SiPkgAttrFuncInputView::OutputSocket { name, .. } => {
//             FuncArgument::find_by_name_for_func(ctx, name, func_id)
//                 .await?
//                 .ok_or(PkgError::MissingFuncArgument(name.to_owned(), func_id))?
//         }
//     };

//     if apa.func_argument_id() != *arg.id() {
//         apa.set_func_argument_id(ctx, arg.id()).await?;
//     }

//     match input {
//         SiPkgAttrFuncInputView::Prop { prop_path, .. } => {
//             let prop = Prop::find_prop_by_path(ctx, schema_variant_id, &prop_path.into()).await?;
//             let prop_ip = InternalProvider::find_for_prop(ctx, *prop.id())
//                 .await?
//                 .ok_or(PkgError::MissingInternalProviderForProp(*prop.id()))?;

//             if apa.internal_provider_id() != *prop_ip.id() {
//                 apa.set_internal_provider_id_safe(ctx, *prop_ip.id())
//                     .await?;
//             }
//         }
//         SiPkgAttrFuncInputView::InputSocket { socket_name, .. } => {
//             let explicit_ip = InternalProvider::find_explicit_for_schema_variant_and_name(
//                 ctx,
//                 schema_variant_id,
//                 &socket_name,
//             )
//             .await?
//             .ok_or(PkgError::MissingInternalProviderForSocketName(
//                 socket_name.to_owned(),
//             ))?;

//             if apa.internal_provider_id() != *explicit_ip.id() {
//                 apa.set_internal_provider_id_safe(ctx, *explicit_ip.id())
//                     .await?;
//             }
//         }
//         _ => {}
//     }

//     Ok(())
// }

#[derive(Debug, Clone)]
pub enum AttrFuncContext {
    Prop(PropId),
    OutputSocket(OutputSocketId),
}

#[allow(clippy::too_many_arguments)]
async fn import_attr_func(
    ctx: &DalContext,
    change_set_pk: Option<ChangeSetPk>,
    context: AttrFuncContext,
    key: Option<String>,
    schema_variant_id: SchemaVariantId,
    func_id: FuncId,
    inputs: Vec<SiPkgAttrFuncInputView>,
    _thing_map: &mut ThingMap,
) -> PkgResult<()> {
    let prototype_id = get_prototype_for_context(ctx, context, key).await?;

    let prototype_func_id = AttributePrototype::func_id(ctx, prototype_id).await?;

    if prototype_func_id != func_id {
        AttributePrototype::update_func_by_id(ctx, prototype_id, func_id).await?;
    }

    for input in &inputs {
        match change_set_pk {
            None => {
                create_attr_proto_arg(ctx, prototype_id, input, func_id, schema_variant_id).await?;
            }
            Some(_) => {
                todo!();
                //                let (unique_id, deleted) = match input {
                //                    SiPkgAttrFuncInputView::Prop {
                //                        unique_id, deleted, ..
                //                    }
                //                    | SiPkgAttrFuncInputView::InputSocket {
                //                        unique_id, deleted, ..
                //                    }
                //                    | SiPkgAttrFuncInputView::OutputSocket {
                //                        unique_id, deleted, ..
                //                    } => (
                //                        unique_id
                //                            .as_deref()
                //                            .ok_or(PkgError::MissingUniqueIdForNode("attr-func-input".into()))?,
                //                        *deleted,
                //                    ),
                //                };
                //
                //                let apa = match thing_map.get(change_set_pk, &unique_id.to_owned()) {
                //                    Some(Thing::AttributePrototypeArgument(apa)) => {
                //                        let mut apa = apa.to_owned();
                //                        if deleted {
                //                            apa.delete_by_id(ctx).await?;
                //                        } else {
                //                            update_attr_proto_arg(
                //                                ctx,
                //                                &mut apa,
                //                                *prototype.id(),
                //                                input,
                //                                func_id,
                //                                schema_variant_id,
                //                            )
                //                            .await?;
                //                        }
                //
                //                        Some(apa)
                //                    }
                //                    _ => {
                //                        if deleted {
                //                            None
                //                        } else {
                //                            Some(
                //                                create_attr_proto_arg(
                //                                    ctx,
                //                                    *prototype.id(),
                //                                    input,
                //                                    func_id,
                //                                    schema_variant_id,
                //                                )
                //                                .await?,
                //                            )
                //                        }
                // }
                //                };

                //                if let Some(apa) = apa {
                //                    thing_map.insert(
                //                        change_set_pk,
                //                        unique_id.to_owned(),
                //                        Thing::AttributePrototypeArgument(apa),
                //                    );
                //                }
            }
        }
    }

    Ok(())
}

fn prop_kind_for_pkg_prop(pkg_prop: &SiPkgProp<'_>) -> PropKind {
    match pkg_prop {
        SiPkgProp::Array { .. } => PropKind::Array,
        SiPkgProp::Boolean { .. } => PropKind::Boolean,
        SiPkgProp::Map { .. } => PropKind::Map,
        SiPkgProp::Number { .. } => PropKind::Integer,
        SiPkgProp::Object { .. } => PropKind::Object,
        SiPkgProp::String { .. } => PropKind::String,
    }
}

async fn create_dal_prop(
    ctx: &DalContext,
    data: &SiPkgPropData,
    kind: PropKind,
    schema_variant_id: SchemaVariantId,
    parent_prop_info: Option<ParentPropInfo>,
) -> PkgResult<Prop> {
    let prop_parent = match parent_prop_info {
        None => PropParent::SchemaVariant(schema_variant_id),
        Some(parent_info) => {
            if parent_info.kind.ordered() {
                PropParent::OrderedProp(parent_info.prop_id)
            } else {
                PropParent::Prop(parent_info.prop_id)
            }
        }
    };

    let prop = Prop::new(
        ctx,
        &data.name,
        kind,
        data.hidden,
        data.doc_link.as_ref().map(|l| l.to_string()),
        Some(((&data.widget_kind).into(), data.widget_options.to_owned())),
        prop_parent,
    )
    .await
    .map_err(SiPkgError::visit_prop)?;

    Ok(prop)
}

#[derive(Debug, Clone)]
struct ParentPropInfo {
    prop_id: PropId,
    path: PropPath,
    kind: PropKind,
}

async fn create_prop(
    spec: SiPkgProp<'_>,
    parent_prop_info: Option<ParentPropInfo>,
    ctx: &PropVisitContext<'_>,
) -> PkgResult<Option<ParentPropInfo>> {
    let prop = match ctx.change_set_pk {
        None => {
            let data = spec.data().ok_or(PkgError::DataNotFound("prop".into()))?;
            create_dal_prop(
                ctx.ctx,
                data,
                prop_kind_for_pkg_prop(&spec),
                ctx.schema_variant_id,
                parent_prop_info,
            )
            .await?
        }
        Some(_) => {
            let parent_path = parent_prop_info
                .as_ref()
                .map(|info| info.path.to_owned())
                .unwrap_or(PropPath::new(["root"]));

            let path = parent_path.join(&PropPath::new([spec.name()]));

            match Prop::find_prop_id_by_path_opt(ctx.ctx, ctx.schema_variant_id, &path).await? {
                None => {
                    let data = spec.data().ok_or(PkgError::DataNotFound("prop".into()))?;
                    create_dal_prop(
                        ctx.ctx,
                        data,
                        prop_kind_for_pkg_prop(&spec),
                        ctx.schema_variant_id,
                        parent_prop_info,
                    )
                    .await?
                }
                Some(prop_id) => Prop::get_by_id(ctx.ctx, prop_id).await?,
            }
        }
    };

    let prop_id = prop.id();

    // Both attribute functions and default values have to be set *after* the schema variant is
    // "finalized", so we can't do until we construct the *entire* prop tree. Hence we push work
    // queues up to the outer context via the PropVisitContext, which uses Mutexes for interior
    // mutability (maybe there's a better type for that here?)

    if let Some(data) = spec.data() {
        if let Some(default_value_info) = match &spec {
            SiPkgProp::String { .. } => {
                if let Some(serde_json::Value::String(default_value)) = &data.default_value {
                    Some(DefaultValueInfo::String {
                        prop_id,
                        default_value: default_value.to_owned(),
                    })
                } else {
                    // Raise error here for type mismatch
                    None
                }
            }
            SiPkgProp::Number { .. } => {
                if let Some(serde_json::Value::Number(default_value_number)) = &data.default_value {
                    if default_value_number.is_i64() {
                        default_value_number
                            .as_i64()
                            .map(|dv_i64| DefaultValueInfo::Number {
                                prop_id,
                                default_value: dv_i64,
                            })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            SiPkgProp::Boolean { .. } => {
                if let Some(serde_json::Value::Bool(default_value)) = &data.default_value {
                    Some(DefaultValueInfo::Boolean {
                        prop_id,
                        default_value: *default_value,
                    })
                } else {
                    None
                }
            }
            // Default values for complex types are not yet supported in packages
            _ => None,
        } {
            ctx.default_values.lock().await.push(default_value_info);
        }
    }

    if matches!(&spec, SiPkgProp::Map { .. }) {
        for map_key_func in spec.map_key_funcs()? {
            let key = map_key_func.key();
            let mut inputs = map_key_func.inputs()?;
            let func_unique_id = map_key_func.func_unique_id();

            ctx.map_key_funcs.lock().await.push((
                key.to_owned(),
                AttrFuncInfo {
                    func_unique_id: func_unique_id.to_owned(),
                    prop_id,
                    inputs: inputs.drain(..).map(Into::into).collect(),
                },
            ));
        }
    }

    if let Some(func_unique_id) = spec.data().and_then(|data| data.func_unique_id.to_owned()) {
        let mut inputs = spec.inputs()?;
        ctx.attr_funcs.lock().await.push(AttrFuncInfo {
            func_unique_id,
            prop_id,
            inputs: inputs.drain(..).map(Into::into).collect(),
        });
    }

    Ok(Some(ParentPropInfo {
        prop_id: prop.id(),
        path: prop.path(ctx.ctx).await?,
        kind: prop.kind,
    }))
}

pub async fn attach_resource_payload_to_value(
    ctx: &DalContext,
    schema_variant_id: SchemaVariantId,
) -> PkgResult<()> {
    let func_id = Func::find_by_name(ctx, "si:resourcePayloadToValue")
        .await?
        .ok_or(PkgError::FuncNotFoundByName(
            "si:resourcePayloadToValue".into(),
        ))?;

    let func_argument_id = FuncArgument::find_by_name_for_func(ctx, "payload", func_id)
        .await?
        .ok_or(PkgError::FuncArgumentNotFoundByName(
            func_id,
            "payload".into(),
        ))?
        .id;

    let source_prop_id = Prop::find_prop_id_by_path(
        ctx,
        schema_variant_id,
        &PropPath::new(["root", "resource", "payload"]),
    )
    .await?;

    let target_id = {
        let resource_value_prop_id = Prop::find_prop_id_by_path(
            ctx,
            schema_variant_id,
            &PropPath::new(["root", "resource_value"]),
        )
        .await?;

        let prototype_id =
            get_prototype_for_context(ctx, AttrFuncContext::Prop(resource_value_prop_id), None)
                .await?;

        AttributePrototype::update_func_by_id(ctx, prototype_id, func_id).await?;

        prototype_id
    };

    let mut rv_input_apa_id = None;
    for apa_id in AttributePrototypeArgument::list_ids_for_prototype(ctx, target_id).await? {
        if func_argument_id
            == AttributePrototypeArgument::func_argument_id_by_id(ctx, apa_id).await?
        {
            rv_input_apa_id = Some(apa_id);
            break;
        }
    }

    match rv_input_apa_id {
        Some(apa_id) => {
            dbg!("existing apa");
            if !{
                if let Some(ValueSource::Prop(prop_id)) =
                    AttributePrototypeArgument::value_source_by_id(ctx, apa_id).await?
                {
                    prop_id == source_prop_id
                } else {
                    false
                }
            } {
                let apa = AttributePrototypeArgument::get_by_id(ctx, apa_id).await?;
                apa.set_value_from_prop_id(ctx, source_prop_id).await?;
            }
        }
        None => {
            let apa = AttributePrototypeArgument::new(ctx, target_id, func_argument_id).await?;
            apa.set_value_from_prop_id(ctx, source_prop_id).await?;
        }
    }

    Ok(())
}
