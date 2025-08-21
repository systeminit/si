use std::{
    collections::{
        BTreeMap,
        VecDeque,
    },
    sync::Arc,
};

use si_events::{
    ContentHash,
    Timestamp,
};
use si_id::{
    AttributeValueId,
    SchemaVariantId,
    ViewId,
};
use telemetry::prelude::*;

use super::{
    Component,
    ComponentResult,
};
use crate::{
    AttributeValue,
    ComponentError,
    DalContext,
    EdgeWeightKind,
    InputSocket,
    OutputSocket,
    Prop,
    PropKind,
    SchemaVariant,
    action::{
        Action,
        prototype::ActionKind,
    },
    attribute::value::default_subscription::{
        DefaultSubscription,
        DefaultSubscriptionSource,
        detect_possible_default_subscription_for_prop,
    },
    dependency_graph::DependencyGraph,
    diagram::geometry::Geometry,
    layer_db_types::{
        ComponentContent,
        ComponentContentV2,
    },
    validation::ValidationOutput,
    workspace_snapshot::{
        DependentValueRoot,
        node_weight::{
            NodeWeight,
            category_node_weight::CategoryNodeKind,
        },
    },
};

#[derive(Debug, Clone)]
enum PotentialSubscription {
    Existing {
        source_av_id: AttributeValueId,
        dest_av_id: AttributeValueId,
    },
    ArrayElem {
        source_av_id: AttributeValueId,
        parent_av_id: AttributeValueId,
    },
}

impl PotentialSubscription {
    pub async fn subscribe(&self, ctx: &DalContext) -> ComponentResult<()> {
        let (source_av_id, dest_av_id) = match self {
            PotentialSubscription::Existing {
                source_av_id,
                dest_av_id,
            } => (*source_av_id, *dest_av_id),
            PotentialSubscription::ArrayElem {
                source_av_id,
                parent_av_id,
            } => {
                let dest_av_id = AttributeValue::insert(ctx, *parent_av_id, None, None).await?;
                (*source_av_id, dest_av_id)
            }
        };

        let default_sub = DefaultSubscription {
            source_av_id,
            dest_av_id,
        };

        default_sub.subscribe(ctx).await?;

        Ok(())
    }
}

impl Component {
    pub async fn new_component_inner(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
        content_address: ContentHash,
        default_sources: Option<&[DefaultSubscriptionSource]>,
    ) -> ComponentResult<Self> {
        let name: String = name.into();

        let component_schema_name =
            SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id)
                .await?
                .name()
                .to_owned();

        let mut component_av_graph = DependencyGraph::new();

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let id = workspace_snapshot.generate_ulid().await?;
        let lineage_id = workspace_snapshot.generate_ulid().await?;

        let node_weight = NodeWeight::new_component(id, lineage_id, content_address);

        // Attach component to category and add use edge to schema variant
        workspace_snapshot.add_or_replace_node(node_weight).await?;

        // Root --> Component Category --> Component (this)
        let component_category_id = workspace_snapshot
            .get_category_node_or_err(CategoryNodeKind::Component)
            .await?;
        Self::add_category_edge(
            ctx,
            component_category_id,
            id.into(),
            EdgeWeightKind::new_use(),
        )
        .await?;

        // Create attribute values for all socket corresponding to input and output sockets.
        for input_socket_id in
            InputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, input_socket_id, Some(id.into()), None, None).await?;

            DependentValueRoot::add_dependent_value_root(
                ctx,
                DependentValueRoot::Unfinished(attribute_value.id().into()),
            )
            .await?;
        }
        for output_socket_id in
            OutputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?
        {
            let attribute_value =
                AttributeValue::new(ctx, output_socket_id, Some(id.into()), None, None).await?;

            DependentValueRoot::add_dependent_value_root(
                ctx,
                DependentValueRoot::Unfinished(attribute_value.id().into()),
            )
            .await?;
        }

        // Walk all the props for the schema variant and create attribute values for all of them
        let root_prop_id = SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;
        let mut work_queue = VecDeque::from([(root_prop_id, None::<AttributeValueId>, None)]);
        let mut default_subscriptions = BTreeMap::new();

        while let Some((prop_id, maybe_parent_attribute_value_id, key)) = work_queue.pop_front() {
            // If we came in with a key, we're the child of a map. We should not descend deeper
            // into it because the value should be governed by its prototype function and will
            // create child values when that function is executed
            let should_descend = key.is_none();

            let prop_weight = workspace_snapshot
                .get_node_weight(prop_id)
                .await?
                .get_prop_node_weight()?;

            let prop_kind = prop_weight.kind();

            // Create an attribute value for the prop.
            let attribute_value = AttributeValue::new(
                ctx,
                prop_id,
                Some(id.into()),
                maybe_parent_attribute_value_id,
                key.clone(),
            )
            .await?;

            component_av_graph.add_id(attribute_value.id());
            if let Some(parent_av_id) = maybe_parent_attribute_value_id {
                component_av_graph.id_depends_on(parent_av_id, attribute_value.id());
            }

            if let Some(source_av_id) = detect_possible_default_subscription_for_prop(
                ctx,
                component_schema_name.clone(),
                prop_id,
                default_sources,
            )
            .await?
            {
                let dest_av_id = attribute_value.id();
                default_subscriptions.insert(
                    dest_av_id,
                    PotentialSubscription::Existing {
                        source_av_id,
                        dest_av_id,
                    },
                );
            }

            if ValidationOutput::get_format_for_attribute_value_id(ctx, attribute_value.id())
                .await?
                .is_some()
            {
                ctx.enqueue_compute_validations(attribute_value.id())
                    .await?;
            }

            match prop_kind {
                PropKind::Object => {
                    if should_descend {
                        let ordered_children = workspace_snapshot
                            .ordered_children_for_node(prop_id)
                            .await?
                            .ok_or(ComponentError::ObjectPropHasNoOrderingNode(prop_id))?;

                        for child_prop_id in ordered_children {
                            work_queue.push_back((
                                child_prop_id.into(),
                                Some(attribute_value.id()),
                                None,
                            ));
                        }
                    }
                }
                PropKind::Map => {
                    let element_prop_id = Prop::direct_single_child_prop_id(ctx, prop_id).await?;

                    if should_descend {
                        for (key, _) in Prop::prototypes_by_key(ctx, element_prop_id).await? {
                            if key.is_some() {
                                work_queue.push_back((
                                    element_prop_id,
                                    Some(attribute_value.id()),
                                    key,
                                ))
                            }
                        }
                    }
                }
                PropKind::Array => {
                    let element_prop_id = Prop::direct_single_child_prop_id(ctx, prop_id).await?;

                    if let Some(source_av_id) = detect_possible_default_subscription_for_prop(
                        ctx,
                        component_schema_name.clone(),
                        element_prop_id,
                        default_sources,
                    )
                    .await?
                    {
                        let parent_av_id = attribute_value.id();
                        default_subscriptions.insert(
                            parent_av_id,
                            PotentialSubscription::ArrayElem {
                                source_av_id,
                                parent_av_id,
                            },
                        );
                    }
                }
                // We want to only add leaves to the DVU roots
                _ => {
                    DependentValueRoot::add_dependent_value_root(
                        ctx,
                        DependentValueRoot::Unfinished(attribute_value.id().into()),
                    )
                    .await?;
                }
            }
        }

        let (node_weight, content) = Self::get_node_weight_and_content(ctx, id.into()).await?;
        let component = Self::assemble(&node_weight, content);

        // Component (this) --> Schema Variant
        Component::add_edge_to_schema_variant(
            ctx,
            component.id,
            schema_variant_id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        component.set_name(ctx, &name).await?;

        //set a component specific prototype for the root/si/type as we don't want it to ever change other than manually
        let sv_type = SchemaVariant::get_by_id(ctx, schema_variant_id)
            .await?
            .get_type(ctx)
            .await?;
        if let Some(sv_type) = sv_type {
            component.set_type(ctx, sv_type).await?;
        }

        // Create all the default subscriptions discovered while creating the
        // component attribute tree
        for (dest_av_id, potential_sub) in &default_subscriptions {
            // Use the dep graph to determine if a parent value of the
            // destination attribute has a subscription or is set by a dynamic
            // function
            let all_parents_of = component_av_graph.all_parents_of(*dest_av_id);
            let mut has_parent_with_sub_or_dynamic_func = false;
            for parent_av_id in all_parents_of {
                if default_subscriptions.contains_key(&parent_av_id) {
                    has_parent_with_sub_or_dynamic_func = true;
                    break;
                }

                if AttributeValue::is_set_by_dependent_function(ctx, parent_av_id).await? {
                    has_parent_with_sub_or_dynamic_func = true;
                    break;
                }
            }

            if has_parent_with_sub_or_dynamic_func {
                continue;
            }

            potential_sub.subscribe(ctx).await?;
        }

        // Find all create action prototypes for the variant and create actions for them.
        for prototype_id in SchemaVariant::find_action_prototypes_by_kind(
            ctx,
            schema_variant_id,
            ActionKind::Create,
        )
        .await?
        {
            Action::new(ctx, prototype_id, Some(component.id)).await?;
        }

        // Update the prop suggestion cache's mapping between schemas and
        // components without recalculating the whole cache
        ctx.workspace_snapshot()?
            .prop_suggestions_cache_no_populate(ctx)
            .await?
            .add_component(ctx, component.id)
            .await?;

        Ok(component)
    }

    /// Create new component node but retain existing content address
    /// This is used to create the replacement nodes on upgrade, so geometries for it need
    /// to be created by hand. Anywhere else you want to use [Self::new](Self::new)
    pub async fn new_with_content_address_and_no_geometry_no_default_subscriptions(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
        content_address: ContentHash,
    ) -> ComponentResult<Self> {
        Self::new_component_inner(ctx, name, schema_variant_id, content_address, None).await
    }

    #[instrument(
        name = "component.new",
        level = "info",
        skip_all,
        fields(
            schema_variant.id = ?schema_variant_id
        ))]
    pub async fn new(
        ctx: &DalContext,
        name: impl Into<String>,
        schema_variant_id: SchemaVariantId,
        view_id: ViewId,
    ) -> ComponentResult<Self> {
        let content = ComponentContentV2 {
            timestamp: Timestamp::now(),
        };

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(ComponentContent::V2(content.clone()).into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let sources = AttributeValue::get_default_subscription_sources(ctx).await?;
        let component = Self::new_component_inner(
            ctx,
            name,
            schema_variant_id,
            hash,
            if sources.is_empty() {
                None
            } else {
                Some(sources.as_slice())
            },
        )
        .await?;

        // Create geometry node
        Geometry::new_for_component(ctx, component.id, view_id).await?;

        Ok(component)
    }
}
