use std::collections::VecDeque;

use serde::Deserialize;
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
    SchemaVariant,
    attribute::path::AttributePath,
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PropSuggestion {
    pub schema: String,
    pub prop: String,
}

#[derive(Debug, Clone)]
pub struct DefaultSubscriptionSource {
    pub av_id: AttributeValueId,
    pub prop: Prop,
    pub schema_name: String,
    pub path: String,
    pub suggest_as_source_for: Vec<PropSuggestion>,
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

pub async fn detect_possible_default_subscription_for_prop(
    ctx: &DalContext,
    dest_schema_name: String,
    dest_prop_id: PropId,
    default_subscription_sources: Option<&[DefaultSubscriptionSource]>,
) -> AttributeValueResult<Option<AttributeValueId>> {
    let Some(default_subscription_sources) = default_subscription_sources else {
        return Ok(None);
    };
    if default_subscription_sources.is_empty() {
        return Ok(None);
    }

    let dest_prop = Prop::get_by_id(ctx, dest_prop_id).await?;
    let dest_prop_path = dest_prop.path(ctx).await?.as_prop_suggestion_path();
    let suggest_sources_for = dest_prop.suggested_sources_for()?;
    let mut default_subscription = None;

    let dest_as_prop_suggestion = PropSuggestion {
        schema: dest_schema_name,
        prop: dest_prop_path,
    };

    for default_source in default_subscription_sources {
        let mut matches = default_source
            .suggest_as_source_for
            .contains(&dest_as_prop_suggestion);

        if !matches {
            matches = suggest_sources_for.contains(&PropSuggestion {
                schema: default_source.schema_name.clone(),
                prop: default_source.path.clone(),
            });
        }

        if !matches {
            continue;
        }

        let source_prop = &default_source.prop;

        let scalar_match = matches!(
            source_prop.kind,
            PropKind::Boolean | PropKind::Integer | PropKind::String
        ) && source_prop.kind == dest_prop.kind;

        if scalar_match || source_prop.is_same_type_as(ctx, &dest_prop).await? {
            if default_subscription.is_some() {
                // Ambiguous match!
                default_subscription = None;
                break;
            }

            default_subscription = Some(default_source.av_id);
        }
    }

    Ok(default_subscription)
}

pub async fn detect_possible_default_connections(
    ctx: &DalContext,
    destination_component_id: ComponentId,
) -> AttributeValueResult<Vec<DefaultSubscription>> {
    let default_subscription_sources =
        AttributeValue::get_default_subscription_sources(ctx).await?;
    if default_subscription_sources.is_empty() {
        return Ok(vec![]);
    }

    let mut result = vec![];

    let dest_schema_variant_id =
        Component::schema_variant_id(ctx, destination_component_id).await?;
    let dest_schema_name = SchemaVariant::schema_for_schema_variant_id(ctx, dest_schema_variant_id)
        .await
        .map_err(Box::new)?
        .name()
        .to_owned();

    let root_av_id = Component::root_attribute_value_id(ctx, destination_component_id).await?;
    let root_children = AttributeValue::child_av_ids(ctx, root_av_id).await?;
    let mut work_queue = VecDeque::from(root_children);
    while let Some(dest_av_id) = work_queue.pop_front() {
        let children = AttributeValue::child_av_ids(ctx, dest_av_id).await?;
        for child_av_id in children {
            work_queue.push_back(child_av_id);
        }

        let dest_prop_id = AttributeValue::prop_id(ctx, dest_av_id).await?;

        if let Some(default_source) = detect_possible_default_subscription_for_prop(
            ctx,
            dest_schema_name.to_owned(),
            dest_prop_id,
            Some(default_subscription_sources.as_slice()),
        )
        .await?
        {
            result.push(DefaultSubscription {
                source_av_id: default_source,
                dest_av_id,
            });
        }
    }

    Ok(result)
}

impl AttributeValue {
    pub async fn remove_default_subscription_source(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<()> {
        let category_id = get_or_create_default_subscription_category(ctx).await?;

        ctx.workspace_snapshot()?
            .remove_edge(
                id,
                category_id,
                EdgeWeightKindDiscriminants::DefaultSubscriptionSource,
            )
            .await?;

        Ok(())
    }

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

    pub async fn is_default_subscription_source(
        ctx: &DalContext,
        id: AttributeValueId,
    ) -> AttributeValueResult<bool> {
        let category_id = get_or_create_default_subscription_category(ctx).await?;

        Ok(ctx
            .workspace_snapshot()?
            .find_edge(
                id,
                category_id,
                EdgeWeightKindDiscriminants::DefaultSubscriptionSource,
            )
            .await
            .is_some())
    }

    pub async fn get_default_subscription_sources(
        ctx: &DalContext,
    ) -> AttributeValueResult<Vec<DefaultSubscriptionSource>> {
        let mut result = vec![];
        let snapshot = ctx.workspace_snapshot()?;

        let Some(default_subscription_category_id) = snapshot
            .get_category_node(CategoryNodeKind::DefaultSubscriptionSources)
            .await?
        else {
            return Ok(vec![]);
        };

        let source_attribute_values: Vec<AttributeValueId> = snapshot
            .incoming_sources_for_edge_weight_kind(
                default_subscription_category_id,
                EdgeWeightKindDiscriminants::DefaultSubscriptionSource,
            )
            .await?
            .iter()
            .copied()
            .map(Into::into)
            .collect();

        for av_id in source_attribute_values {
            let prop_id = AttributeValue::prop_id(ctx, av_id).await?;
            let prop = Prop::get_by_id(ctx, prop_id).await?;
            let path = Prop::path_by_id(ctx, prop_id)
                .await?
                .as_prop_suggestion_path();
            let Some(schema_variant_id) = Prop::schema_variant_id(ctx, prop_id).await? else {
                continue;
            };
            let schema_name = SchemaVariant::schema_for_schema_variant_id(ctx, schema_variant_id)
                .await
                .map_err(Box::new)?
                .name()
                .to_owned();
            let suggest_as_source_for = prop.suggested_as_source_for()?;

            result.push(DefaultSubscriptionSource {
                av_id,
                schema_name,
                path,
                suggest_as_source_for,
                prop,
            });
        }

        Ok(result)
    }
}
