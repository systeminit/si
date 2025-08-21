use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
    },
    time::Instant,
};

use dashmap::DashMap;
use itertools::Itertools;
use serde::Serialize;
use si_id::{
    AttributeValueId,
    ComponentId,
    SchemaVariantId,
    WorkspacePk,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributeValue,
    Component,
    ComponentError,
    DalContext,
    Prop,
    SchemaVariant,
    SchemaVariantError,
    WorkspaceSnapshotError,
    attribute::{
        attributes::{
            AttributeValueIdent,
            Source,
        },
        path::AttributePath,
        value::{
            default_subscription::PropSuggestion,
            subscription::ValueSubscription,
        },
    },
    component::ComponentResult,
    prop::PropError,
    workspace_snapshot::node_weight::reason_node_weight::Reason,
};
#[remain::sorted]
#[derive(Debug, Error)]
pub enum PropSuggestionCacheError {
    #[error("Component error: {0}")]
    Component(#[from] Box<ComponentError>),
    #[error("Prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("Schema Variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("WorkspaceSnapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
}
pub type PropSuggestionCacheResult<T> = Result<T, PropSuggestionCacheError>;

impl From<ComponentError> for PropSuggestionCacheError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}
impl From<PropError> for PropSuggestionCacheError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}
impl From<SchemaVariantError> for PropSuggestionCacheError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}
impl From<WorkspaceSnapshotError> for PropSuggestionCacheError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}

/// Cache for Prop suggestions and Components by Schema Name
/// This is used for each DalCtx lifetime and is helpful particularly during
/// Discovery as we will autosubscribe multiple components in that case
/// When we need longer lived/more shared caching, we can move this elsewhere
#[derive(Debug, Clone, Default)]
pub struct PropSuggestionsCache {
    /// Map of schema variant ID to its suggestion patterns
    pub schema_suggestions: DashMap<SchemaVariantId, SchemaSuggestionMap>,
    /// Map of schema name to components for efficient lookup
    pub schema_to_components: DashMap<String, Vec<ComponentId>>,
}

/// Cache of prop suggestion patterns for a specific schema variant
#[derive(Debug, Clone)]
pub struct SchemaSuggestionMap {
    /// Map of prop path -> list of prop suggestions for that path (suggestSources)
    pub suggest_sources_map: HashMap<String, Vec<PropSuggestion>>,
    /// Map of (target_schema, target_prop) -> list of source prop paths (suggestAsSourceFor)
    pub suggest_as_source_for_map: HashMap<PropSuggestion, Vec<String>>,
}

impl PropSuggestionsCache {
    /// Populate this cache instance with data from the database
    /// Not sure how noisy this will be but I'd like to get the timings in the short term
    #[instrument(level = "info", name = "dal.prop_suggestion_cache.populate", skip_all, 
    fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
        ))]
    pub async fn populate(&self, ctx: &DalContext) -> PropSuggestionCacheResult<()> {
        let span = Span::current();
        span.record("si.change_set.id", ctx.change_set_id().to_string());
        span.record(
            "si.workspace.id",
            ctx.tenancy()
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .to_string(),
        );
        // Build schema_to_components map
        for component_id in Component::list_ids(ctx).await? {
            let comp_schema = Component::schema_for_component_id(ctx, component_id).await?;
            let schema_name = comp_schema.name().to_string();

            // Update or insert the component list for this schema
            self.schema_to_components
                .entry(schema_name)
                .or_default()
                .push(component_id);
        }

        // Build schema suggestions map for each unique schema variant
        let mut processed_schema_variants = HashSet::new();

        for component_id in Component::list_ids(ctx).await? {
            let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

            if processed_schema_variants.contains(&schema_variant_id) {
                continue; // Already processed this schema variant
            }
            processed_schema_variants.insert(schema_variant_id);

            // Build suggestion map for this schema variant
            let mut suggest_sources_map = HashMap::new();
            let mut suggest_as_source_for_map = HashMap::new();

            let all_props = SchemaVariant::all_props(ctx, schema_variant_id).await?;

            for prop in all_props {
                let prop_path = prop.path(ctx).await?.as_prop_suggestion_path();

                // Store suggestSources patterns
                let suggest_sources = prop.suggested_sources_for()?;
                if !suggest_sources.is_empty() {
                    suggest_sources_map.insert(prop_path.clone(), suggest_sources);
                }

                // Store suggestAsSourceFor patterns
                let suggest_as_source_for = prop.suggested_as_source_for()?;
                for suggestion in suggest_as_source_for {
                    suggest_as_source_for_map
                        .entry(suggestion)
                        .or_insert_with(Vec::new)
                        .push(prop_path.clone());
                }
            }

            // Store in cache
            let suggestion_map = SchemaSuggestionMap {
                suggest_sources_map,
                suggest_as_source_for_map,
            };
            self.schema_suggestions
                .insert(schema_variant_id, suggestion_map);
        }
        Ok(())
    }
}

/// Result of an autosubscribe operation with improved structure and context
#[derive(Debug, Clone, Default)]
pub struct AutosubscribeResult {
    pub successful: Vec<SuccessfulSubscription>,
    pub conflicts: Vec<ConflictedSubscription>,
    pub errors: Vec<SubscriptionError>,
}

/// A successfully created subscription with context
#[derive(Debug, Clone, Serialize)]
pub struct SuccessfulSubscription {
    pub target_path: AttributePath,
    pub subscription_source: Source,
    pub matched_value: serde_json::Value,
}

/// A subscription that couldn't be created due to conflicts
#[derive(Debug, Clone, Serialize)]
pub struct ConflictedSubscription {
    pub target_path: AttributePath,
    pub matches: Vec<SubscriptionMatch>,
}

/// A potential subscription match
#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionMatch {
    pub component_id: ComponentId,
    pub source_path: AttributePath,
    pub value: serde_json::Value,
}

/// An error that occurred during subscription creation
#[derive(Debug, Clone)]
pub struct SubscriptionError {
    pub target_path: AttributePath,
    pub error: String,
    pub attempted_source: Option<(ComponentId, AttributePath)>,
}

/// Context object to hold shared data during autosubscribe operations
#[derive(Debug)]
struct AutosubscribeContext {
    component_id: ComponentId,
    schema_name: String,
    schema_variant_id: SchemaVariantId,
    attribute_values: Vec<AttributeValueId>,
}

impl AutosubscribeResult {
    /// Number of successful subscriptions created
    pub fn success_count(&self) -> usize {
        self.successful.len()
    }

    /// Number of conflicted subscriptions that need user intervention
    pub fn conflict_count(&self) -> usize {
        self.conflicts.len()
    }

    /// Number of errors that occurred
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Whether there are any issues that need attention
    pub fn has_issues(&self) -> bool {
        !self.conflicts.is_empty() || !self.errors.is_empty()
    }

    /// Human-readable summary of the operation
    pub fn summary(&self) -> String {
        format!(
            "Created {} subscriptions, {} conflicts, {} errors",
            self.success_count(),
            self.conflict_count(),
            self.error_count()
        )
    }

    /// Get all successful subscriptions for a specific target path
    pub fn successful_for_path(&self, path: &AttributePath) -> Vec<&SuccessfulSubscription> {
        self.successful
            .iter()
            .filter(|s| &s.target_path == path)
            .collect()
    }

    /// Get all conflicts for a specific target path
    pub fn conflicts_for_path(&self, path: &AttributePath) -> Vec<&ConflictedSubscription> {
        self.conflicts
            .iter()
            .filter(|c| &c.target_path == path)
            .collect()
    }
}

impl AutosubscribeContext {
    /// Create a new context for autosubscribe operations
    async fn new(ctx: &DalContext, component_id: ComponentId) -> ComponentResult<Self> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let schema = component.schema(ctx).await?;
        let schema_name = schema.name().to_string();
        let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;

        // Get all attribute values for this component
        let attribute_value_tree = AttributeValue::tree_for_component(ctx, component_id).await?;
        let mut flattened = Vec::new();
        let mut queue = VecDeque::from_iter(attribute_value_tree.keys().copied());

        while let Some(av_id) = queue.pop_front() {
            flattened.push(av_id);
            if let Some(children) = attribute_value_tree.get(&av_id) {
                queue.extend(children);
            }
        }

        Ok(Self {
            component_id,
            schema_name,
            schema_variant_id,
            attribute_values: flattened,
        })
    }

    /// Find potential subscription matches based on prop suggestions within this component
    async fn find_suggestion_matches(
        &self,
        ctx: &DalContext,
        cache: &PropSuggestionsCache,
        potential_matches: &mut HashMap<AttributeValueId, HashSet<PotentialSource>>,
        potential_match_values: &mut HashMap<AttributeValueId, (AttributePath, serde_json::Value)>,
    ) -> ComponentResult<()> {
        // Get cached schema suggestions for this component's schema variant
        let Some(component_suggestion_map) = cache.schema_suggestions.get(&self.schema_variant_id)
        else {
            return Err(ComponentError::PropSuggestionCacheEmpty(self.component_id));
        };

        // Check each attribute value in this component for incoming subscriptions
        for &attribute_value_id in &self.attribute_values {
            // Skip if this attribute value doesn't have a prop or a manually set value
            let Some(prop_id) = AttributeValue::prop_id_opt(ctx, attribute_value_id).await? else {
                continue;
            };
            // Skip if this av is set by a dependent function (which means it's not been set manually)
            if AttributeValue::is_set_by_dependent_function(ctx, attribute_value_id).await? {
                continue;
            }
            let Some(current_value) = AttributeValue::view(ctx, attribute_value_id).await? else {
                continue; // Skip if no value is set
            };

            let prop = Prop::get_by_id(ctx, prop_id).await?;
            let prop_path = prop.path(ctx).await?.as_prop_suggestion_path();
            let prop_suggestion = PropSuggestion {
                schema: self.schema_name.clone(),
                prop: prop_path.clone(),
            };
            let target_path = AttributePath::JsonPointer(prop_path);

            // Check for explicit suggestions from the cache
            if let Some(suggestions) = component_suggestion_map
                .suggest_sources_map
                .get(&prop_suggestion.prop)
            {
                // This prop has explicit suggested sources, check them
                for suggestion in suggestions {
                    if let Some(source_components) =
                        cache.schema_to_components.get(&suggestion.schema)
                    {
                        for &source_component_id in source_components.iter() {
                            if source_component_id == self.component_id {
                                continue; // Skip self
                            }

                            // Find the attribute value at the suggested prop path
                            if let Some(source_av_id) = AttributeValueIdent::new(&suggestion.prop)
                                .resolve(ctx, source_component_id)
                                .await?
                            {
                                let source_value = AttributeValue::view(ctx, source_av_id).await?;
                                // If values match, this is a potential subscription
                                if source_value == Some(current_value.clone()) {
                                    let source = PotentialSource {
                                        component_id: source_component_id,
                                        attribute_value_id: source_av_id,
                                        path: AttributePath::JsonPointer(suggestion.prop.clone()),
                                    };
                                    potential_matches
                                        .entry(attribute_value_id)
                                        .or_default()
                                        .insert(source);
                                    potential_match_values.insert(
                                        attribute_value_id,
                                        (target_path.clone(), current_value.clone()),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Find potential subscription matches based on reverse suggestions (suggestAsSourceFor)
    async fn find_reverse_suggestion_matches(
        &self,
        ctx: &DalContext,
        cache: &PropSuggestionsCache,
        potential_matches: &mut HashMap<AttributeValueId, HashSet<PotentialSource>>,
        potential_match_values: &mut HashMap<AttributeValueId, (AttributePath, serde_json::Value)>,
    ) -> ComponentResult<()> {
        // Check for reverse suggestions (suggestAsSourceFor) from the cache
        for &attribute_value_id in &self.attribute_values {
            let Some(prop_id) = AttributeValue::prop_id_opt(ctx, attribute_value_id).await? else {
                continue;
            };
            if AttributeValue::is_set_by_dependent_function(ctx, attribute_value_id).await? {
                continue;
            }
            let Some(current_value) = AttributeValue::view(ctx, attribute_value_id).await? else {
                continue;
            };

            let prop = Prop::get_by_id(ctx, prop_id).await?;
            let prop_path = prop.path(ctx).await?.as_prop_suggestion_path();
            let prop_suggestion = PropSuggestion {
                schema: self.schema_name.clone(),
                prop: prop_path.clone(),
            };
            let target_path = AttributePath::JsonPointer(prop_path);

            for schema_variant_entry in cache.schema_suggestions.iter() {
                let other_schema_variant_id = schema_variant_entry.key();
                let other_suggestion_map = schema_variant_entry.value();

                // Look for sources that suggest themselves for this schema and prop
                if let Some(source_prop_paths) = other_suggestion_map
                    .suggest_as_source_for_map
                    .get(&prop_suggestion)
                {
                    // Get components for this other schema variant
                    let other_schema_name = if let Ok(other_schema_variant) =
                        SchemaVariant::get_by_id(ctx, *other_schema_variant_id).await
                    {
                        other_schema_variant.schema(ctx).await?.name().to_string()
                    } else {
                        continue;
                    };

                    if let Some(source_components) =
                        cache.schema_to_components.get(&other_schema_name)
                    {
                        for source_prop_path in source_prop_paths {
                            for &source_component_id in source_components.iter() {
                                if source_component_id == self.component_id {
                                    continue; // Skip self
                                }

                                if let Some(source_av_id) =
                                    AttributeValueIdent::new(source_prop_path)
                                        .resolve(ctx, source_component_id)
                                        .await?
                                {
                                    let Some(source_value) =
                                        AttributeValue::view(ctx, source_av_id).await?
                                    else {
                                        continue;
                                    };
                                    // If values match, this is a potential subscription
                                    if source_value == current_value {
                                        let source = PotentialSource {
                                            component_id: source_component_id,
                                            attribute_value_id: source_av_id,
                                            path: AttributePath::JsonPointer(
                                                source_prop_path.clone(),
                                            ),
                                        };
                                        potential_matches
                                            .entry(attribute_value_id)
                                            .or_default()
                                            .insert(source);
                                        potential_match_values.insert(
                                            attribute_value_id,
                                            (target_path.clone(), current_value.clone()),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Process potential matches into final results (subscriptions, conflicts, errors)
    async fn process_matches(
        &self,
        ctx: &DalContext,
        potential_matches: HashMap<AttributeValueId, HashSet<PotentialSource>>,
        potential_match_values: HashMap<AttributeValueId, (AttributePath, serde_json::Value)>,
        result: &mut AutosubscribeResult,
    ) -> ComponentResult<()> {
        // Process each potential match
        for (dest_av_id, matches) in potential_matches {
            let (target_path, stored_value) = potential_match_values
                .get(&dest_av_id)
                .cloned()
                .unwrap_or_else(|| {
                    (
                        AttributePath::JsonPointer("".to_string()),
                        serde_json::Value::Null,
                    )
                });

            if let Ok(source) = matches.iter().exactly_one() {
                // Unambiguous match - create the subscription
                let (root_attribute_value_id, source_path) =
                    AttributeValue::path_from_root(ctx, source.attribute_value_id).await?;
                let subscription = ValueSubscription {
                    attribute_value_id: root_attribute_value_id,
                    path: AttributePath::from_json_pointer(&source_path),
                };

                // Make sure the subscribed-to path is valid
                match subscription.validate(ctx).await {
                    Ok(_) => {
                        // Create the subscription
                        match AttributeValue::set_to_subscription(
                            ctx,
                            dest_av_id,
                            subscription.clone(),
                            None,
                            Reason::Autosubscription,
                        )
                        .await
                        {
                            Ok(_) => {
                                result.successful.push(SuccessfulSubscription {
                                    target_path,
                                    subscription_source: Source::Subscription {
                                        component: source.component_id.into(),
                                        path: source_path,
                                        func: None,
                                        _keep_existing_subscriptions: None,
                                    },
                                    matched_value: stored_value,
                                });
                            }
                            Err(err) => {
                                result.errors.push(SubscriptionError {
                                    target_path,
                                    error: format!("Failed to create subscription: {err}"),
                                    attempted_source: Some((
                                        source.component_id,
                                        source.path.clone(),
                                    )),
                                });
                            }
                        }
                    }
                    Err(err) => {
                        result.errors.push(SubscriptionError {
                            target_path,
                            error: format!("Invalid subscription path: {err}"),
                            attempted_source: Some((source.component_id, source.path.clone())),
                        });
                    }
                }
            } else {
                // Multiple matches - record as conflict
                let subscription_matches: Vec<SubscriptionMatch> = matches
                    .iter()
                    .map(|source| SubscriptionMatch {
                        component_id: source.component_id,
                        source_path: source.path.clone(),
                        value: stored_value.clone(),
                    })
                    .collect();

                if !subscription_matches.is_empty() {
                    result.conflicts.push(ConflictedSubscription {
                        target_path,
                        matches: subscription_matches,
                    });
                }
            }
        }
        Ok(())
    }
}

/// A potential source for a subscription
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
struct PotentialSource {
    component_id: ComponentId,
    attribute_value_id: AttributeValueId,
    path: AttributePath,
}

impl Component {
    /// Automatically creates prop subscriptions based on prop suggestions and matching values.
    ///
    /// This function mimics the prior behavior of `autoconnect`, but instead of dealing with sockets,
    /// it works with prop subscriptions.
    ///
    /// 1. For each attribute value in the component, if it has a value that's set manually,
    ///    and has a prop suggestion, look for components that match the prop suggestion and
    ///    check if the attribute value's value matches the one we're trying to find subscriptions for.
    /// 2. Find components with prop suggestions (suggest_source_as) that go the other way, and see if those match.
    /// 3. If there's a single match, replace the manually set value with a subscription
    ///    but if there are multiple eligible matches, return them as ambiguous for user decision.
    #[instrument(level = "info", name = "dal.component.autosubscribe_1", skip_all, 
    fields(
            si.change_set.id = Empty,
            si.workspace.id = Empty,
            si.dal.autosubscribe.fetched_cache_ms = Empty,
            si.dal.autosubscribe.suggest_ms = Empty,
            si.dal.autosubscribe.suggest_as_source_for_ms = Empty,
            si.dal.autosubscribe.created_subcriptions = Empty,
            si.dal.autosubscribe.conflicted_subscriptions = Empty,
        ))]
    pub async fn autosubscribe(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentResult<AutosubscribeResult> {
        let span = Span::current();
        let start = Instant::now();
        span.record("si.change_set.id", ctx.change_set_id().to_string());
        span.record(
            "si.workspace.id",
            ctx.tenancy()
                .workspace_pk_opt()
                .unwrap_or(WorkspacePk::NONE)
                .to_string(),
        );

        let mut result = AutosubscribeResult::default();

        // Use the cached prop suggestions from workspace snapshot
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let cache = workspace_snapshot.prop_suggestions_cache(ctx).await?;
        span.record(
            "si.dal.autosubscribe.fetched_cache_ms",
            start.elapsed().as_millis(),
        );

        // Create context for this operation
        let context = AutosubscribeContext::new(ctx, component_id).await?;

        // Build a map of potential matches by destination attribute value
        let mut potential_matches: HashMap<AttributeValueId, HashSet<PotentialSource>> =
            HashMap::new();
        let mut potential_match_values: HashMap<
            AttributeValueId,
            (AttributePath, serde_json::Value),
        > = HashMap::new();

        // 1. Find matches based on explicit prop suggestions
        context
            .find_suggestion_matches(
                ctx,
                cache,
                &mut potential_matches,
                &mut potential_match_values,
            )
            .await?;
        span.record(
            "si.dal.autosubscribe.suggest_ms",
            start.elapsed().as_millis(),
        );

        // 2. Find matches based on reverse suggestions (suggestAsSourceFor)
        context
            .find_reverse_suggestion_matches(
                ctx,
                cache,
                &mut potential_matches,
                &mut potential_match_values,
            )
            .await?;
        span.record(
            "si.dal.autosubscribe.suggest_as_source_for_ms",
            start.elapsed().as_millis(),
        );

        // 3. Process all potential matches into final results
        context
            .process_matches(ctx, potential_matches, potential_match_values, &mut result)
            .await?;

        span.record(
            "si.dal.autosubscribe.created_subcriptions",
            result.success_count(),
        );
        span.record(
            "si.dal.autosubscribe.conflicted_subscriptions",
            result.conflict_count(),
        );

        Ok(result)
    }
}
