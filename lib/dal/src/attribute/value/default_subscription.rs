use std::collections::{
    BTreeMap,
    BTreeSet,
    VecDeque,
};

use si_id::{
    AttributeValueId,
    ComponentId,
    PropId,
    ulid::Ulid,
};

use super::{
    AttributeValue,
    AttributeValueResult,
    subscription::ValueSubscription,
};
use crate::{
    Component,
    DalContext,
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    Prop,
    PropKind,
    Schema,
    SchemaVariant,
    attribute::path::AttributePath,
    prop::ResolvedPropSuggestion,
    schema::variant::SchemaVariantResult,
    workspace_snapshot::node_weight::{
        CategoryNodeWeight,
        NodeWeight,
        category_node_weight::CategoryNodeKind,
    },
};

async fn get_or_create_default_subscription_category(
    ctx: &DalContext,
) -> AttributeValueResult<Ulid> {
    let snapshot = ctx.workspace_snapshot()?;

    Ok(
        match snapshot
            .get_category_node(CategoryNodeKind::DefaultSubscriptionSources)
            .await?
        {
            Some(id) => id,
            None => {
                let static_id = CategoryNodeKind::DefaultSubscriptionSources
                    .static_id()
                    .unwrap_or_default();

                let node_weight = CategoryNodeWeight::new(
                    static_id,
                    static_id,
                    CategoryNodeKind::DefaultSubscriptionSources,
                );
                snapshot
                    .add_or_replace_node(NodeWeight::Category(node_weight))
                    .await?;
                let root_id = snapshot.root().await?;
                snapshot
                    .add_edge(
                        root_id,
                        EdgeWeight::new(EdgeWeightKind::new_use()),
                        static_id,
                    )
                    .await?;

                static_id
            }
        },
    )
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct DefaultSubscription {
    pub source_av_id: AttributeValueId,
    pub dest_av_id: AttributeValueId,
}

impl DefaultSubscription {
    /// NOTE: This will clobber existing subscriptions
    pub async fn subscribe(&self, ctx: &DalContext) -> AttributeValueResult<()> {
        let (root_attribute_value_id, path) =
            AttributeValue::path_from_root(ctx, self.source_av_id).await?;
        let subscription = ValueSubscription {
            attribute_value_id: root_attribute_value_id,
            path: AttributePath::from_json_pointer(path),
        };

        AttributeValue::set_to_subscriptions(ctx, self.dest_av_id, vec![subscription], None).await
    }
}

pub async fn detect_possible_default_connections(
    ctx: &DalContext,
    destination_component_id: ComponentId,
    suggestions: &BTreeMap<PropId, BTreeSet<ResolvedPropSuggestion>>,
) -> AttributeValueResult<Vec<DefaultSubscription>> {
    let source_attribute_values = AttributeValue::get_default_subscription_sources(ctx).await?;
    if source_attribute_values.is_empty() {
        return Ok(vec![]);
    }

    let mut potential_source_props = BTreeMap::new();
    for source_attribute_value_id in source_attribute_values {
        let prop_id = AttributeValue::prop_id(ctx, source_attribute_value_id).await?;
        let prop = Prop::get_by_id(ctx, prop_id).await?;
        // TODO: Support for objects, maps, and arrays will come next,
        // to support them we have to do a deep comparison on the two
        // prop types.
        if !matches!(
            prop.kind,
            PropKind::String | PropKind::Integer | PropKind::Boolean
        ) {
            continue;
        }

        potential_source_props.insert(prop_id, (prop, source_attribute_value_id));
    }

    let mut result = vec![];

    let root_av_id = Component::root_attribute_value_id(ctx, destination_component_id).await?;
    let root_children = AttributeValue::child_av_ids(ctx, root_av_id).await?;
    let mut work_queue = VecDeque::from(root_children);
    while let Some(dest_av_id) = work_queue.pop_front() {
        let dest_prop_id = AttributeValue::prop_id(ctx, dest_av_id).await?;
        let prop = Prop::get_by_id(ctx, dest_prop_id).await?;
        let mut potential = None;

        let children = AttributeValue::child_av_ids(ctx, dest_av_id).await?;
        work_queue.extend(children);

        if let Some(suggestions) = suggestions.get(&dest_prop_id) {
            for &suggestion in suggestions {
                if suggestion.dest_prop_id != dest_prop_id {
                    continue;
                }

                let Some((source_prop, source_av_id)) =
                    potential_source_props.get(&suggestion.source_prop_id)
                else {
                    continue;
                };

                if source_prop.kind == prop.kind {
                    if potential.is_some() {
                        // Ambiguous match!
                        potential = None;
                        break;
                    }

                    potential = Some(DefaultSubscription {
                        source_av_id: *source_av_id,
                        dest_av_id,
                    });
                }
            }
        }

        if let Some(potential) = potential {
            result.push(potential);
        }
    }

    Ok(result)
}

/// Returns a map of potential *destination* prop ids to the
/// ResolvedPropSuggestion for the entire change set
pub async fn calculate_all_prop_suggestions_for_change_set(
    ctx: &DalContext,
) -> SchemaVariantResult<BTreeMap<PropId, BTreeSet<ResolvedPropSuggestion>>> {
    let mut result: BTreeMap<PropId, BTreeSet<ResolvedPropSuggestion>> = BTreeMap::new();

    for schema_id in Schema::list_ids(ctx).await? {
        for variant_id in Schema::list_schema_variant_ids(ctx, schema_id).await? {
            let mut suggestions = vec![];
            suggestions.extend(SchemaVariant::props_suggested_as_sources(ctx, variant_id).await?);
            suggestions
                .extend(SchemaVariant::props_suggested_as_destinations(ctx, variant_id).await?);

            for suggestion in suggestions {
                result
                    .entry(suggestion.dest_prop_id)
                    .and_modify(|sugs| {
                        sugs.insert(suggestion);
                    })
                    .or_insert_with(|| BTreeSet::from([suggestion]));
            }
        }
    }

    Ok(result)
}

impl AttributeValue {
    pub async fn set_as_default_subscription_source(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let category_id = get_or_create_default_subscription_category(ctx).await?;

        AttributeValue::add_default_subscription_source_edge(
            ctx,
            id,
            category_id,
            EdgeWeightKind::DefaultSubscriptionSource,
        )
        .await?;

        Ok(())
    }

    pub async fn get_default_subscription_sources(
        ctx: &DalContext,
    ) -> AttributeValueResult<Vec<AttributeValueId>> {
        let snapshot = ctx.workspace_snapshot()?;

        let Some(default_subscription_category_id) = snapshot
            .get_category_node(CategoryNodeKind::DefaultSubscriptionSources)
            .await?
        else {
            return Ok(vec![]);
        };

        Ok(snapshot
            .incoming_sources_for_edge_weight_kind(
                default_subscription_category_id,
                EdgeWeightKindDiscriminants::DefaultSubscriptionSource,
            )
            .await?
            .iter()
            .copied()
            .map(Into::into)
            .collect())
    }
}
