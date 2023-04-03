use dal::action_prototype::ActionKind;
use dal::component::confirmation::view::{ConfirmationStatus, ConfirmationView, Recommendation};
use dal::{AttributeValueId, ComponentId, DalContext, FixCompletionStatus};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;
use sdf_server::service::dev::CREATE_CONFIRMATION_NAME;
use sdf_server::service::dev::DELETE_CONFIRMATION_NAME;
use sdf_server::service::fix::run::FixRunRequest;
use std::collections::{HashMap, HashSet};

use crate::service_tests::scenario::ScenarioHarness;
use crate::test_setup;

/// Recommendation: run this test with the following environment variable...
/// ```shell
/// SI_TEST_BUILTIN_SCHEMAS=none
/// ```
#[test]
#[ignore]
async fn fix_flow_deletion() {
    test_setup!(
        _sdf_ctx,
        _secret_key,
        _pg,
        _conn,
        _txn,
        _nats_conn,
        _nats,
        veritech,
        encr_key,
        app,
        _nw,
        auth_token,
        octx,
        _job_processor,
        _council_subject_prefix,
    );
    // Just borrow it the whole time because old habits die hard.
    let ctx: &mut DalContext = &mut octx;

    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(ctx, app, auth_token, &[]).await;

    // Author a schema using the appropriate route. We'll add it to our harness' cache afterwards.
    // We'll do this all in a changeset and then apply it.
    assert!(ctx.visibility().is_head());
    harness.create_change_set_and_update_ctx(ctx, "poop1").await;
    let schema_name = "lock//in";
    harness
        .author_single_schema_with_default_variant(ctx, schema_name)
        .await;
    harness.add_schemas(ctx, &[schema_name]).await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(ctx)
        .await;

    // In a new changeset, create nodes and connections. Apply the changeset when finished.
    harness.create_change_set_and_update_ctx(ctx, "poop2").await;
    let boaster = harness.create_node(ctx, schema_name, None).await;
    let derke = harness.create_node(ctx, schema_name, None).await;
    let alfajer = harness.create_node(ctx, schema_name, None).await;
    let leo = harness.create_node(ctx, schema_name, None).await;
    let chronicle = harness.create_node(ctx, schema_name, None).await;
    harness
        .create_connection(ctx, alfajer.node_id, boaster.node_id, "universal")
        .await;
    harness
        .create_connection(ctx, boaster.node_id, leo.node_id, "universal")
        .await;
    harness
        .create_connection(ctx, alfajer.node_id, chronicle.node_id, "universal")
        .await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(ctx)
        .await;

    // On HEAD, check the confirmations and recommendations to see that they match what we expect.
    // We also want to ensure that the recommendations are topologically sorted with stable
    // ordering (i.e. use a "Vec" with non-arbitrary ordering for the assertion(s)).
    let confirmations = harness.list_confirmations(ctx).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    let parsed = ParsedConfirmations::new(confirmations);
    let expected = vec![
        RecommendationMetadata {
            component_id: derke.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: alfajer.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: boaster.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: chronicle.component_id,
            action_kind: ActionKind::Create,
        },
        RecommendationMetadata {
            component_id: leo.component_id,
            action_kind: ActionKind::Create,
        },
    ];
    assert_eq!(
        expected,                            // expected
        parsed.recommendation_metadata_list  // actual
    );

    // As we are checking that the confirmations look as we expect, assemble fix run requests.
    let mut seen_component_ids = HashSet::new();
    let mut assembler = FixRunRequestAssembler::new();
    for ((component_id, approximate_confirmation_kind), parsed_single) in parsed.inner {
        seen_component_ids.insert(component_id);

        match approximate_confirmation_kind {
            ApproximateConfirmationKind::NeedsDestroy => {
                assert_eq!(
                    ConfirmationStatus::Success,       // expected
                    parsed_single.confirmation_status  // actual
                );
                assert!(parsed_single.maybe_recommendation.is_none());
            }
            ApproximateConfirmationKind::NeedsCreate => {
                assert_eq!(
                    ConfirmationStatus::Failure,       // expected
                    parsed_single.confirmation_status  // actual
                );

                let recommendation = parsed_single
                    .maybe_recommendation
                    .expect("recommendation not found");
                assembler.insert(
                    RecommendationMetadata {
                        component_id: recommendation.component_id,
                        action_kind: recommendation.action_kind,
                    },
                    (
                        recommendation.confirmation_attribute_value_id,
                        recommendation.recommended_action,
                    ),
                );
            }
        }
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );
    assert_eq!(
        5,               // expected
        assembler.len()  // actual
    );

    // Run the fixes for the corresponding confirmations. We will use the exact order of the
    // recommendations during assembly.
    let fix_requests = assembler.assemble(&parsed.recommendation_metadata_list);
    assert_eq!(
        5,                  // expected
        fix_requests.len()  // actual
    );
    let create_fix_batch_id = harness.run_fixes(ctx, fix_requests).await;

    // Check that the fix batch succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx).await;
    let create_view = fix_batch_history_views.pop().expect("no fix batches found");
    assert!(fix_batch_history_views.is_empty());
    assert_eq!(
        create_fix_batch_id, // expected
        create_view.id,      // actual
    );
    assert_eq!(
        FixCompletionStatus::Success,                 // expected
        create_view.status.expect("no status found")  // actual
    );

    // Check confirmations again on HEAD. We should have no recommendations this time.
    let confirmations = harness.list_confirmations(ctx).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    let parsed = ParsedConfirmations::new(confirmations);
    assert!(parsed.recommendation_metadata_list.is_empty());

    let mut seen_component_ids = HashSet::new();
    for ((component_id, _), parsed_single) in parsed.inner {
        seen_component_ids.insert(component_id);
        assert_eq!(
            ConfirmationStatus::Success,       // expected
            parsed_single.confirmation_status  // actual
        );
        assert!(parsed_single.maybe_recommendation.is_none());
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );

    // Go back to model, immediately merge and come back. We should still see no recommendations.
    harness.create_change_set_and_update_ctx(ctx, "poop3").await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(ctx)
        .await;
    let confirmations = harness.list_confirmations(ctx).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    let parsed = ParsedConfirmations::new(confirmations);
    assert!(parsed.recommendation_metadata_list.is_empty());

    let mut seen_component_ids = HashSet::new();
    for ((component_id, _), parsed_single) in parsed.inner {
        seen_component_ids.insert(component_id);
        assert_eq!(
            ConfirmationStatus::Success,       // expected
            parsed_single.confirmation_status  // actual
        );
        assert!(parsed_single.maybe_recommendation.is_none());
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );

    // Ensure the resource exists after creation.
    let diagram = harness.get_diagram(ctx).await;
    for component in diagram.components() {
        let maybe_data_raw = component.resource().data.clone().expect("data is empty");
        let data = serde_json::to_string(&maybe_data_raw).expect("could not deserialize data");
        assert_eq!(
            "\"poop\"", // expected
            &data       // actual
        );
    }

    // Refresh the resources. The order at which we refresh should have no
    // effect on the order of the confirmations and recommendations that come back.
    assert!(ctx.visibility().is_head());
    harness.resource_refresh(ctx, boaster.component_id).await;
    harness.resource_refresh(ctx, derke.component_id).await;
    harness.resource_refresh(ctx, leo.component_id).await;
    harness.resource_refresh(ctx, alfajer.component_id).await;
    harness.resource_refresh(ctx, chronicle.component_id).await;

    // Ensure the resource continues to exist after refresh.
    let diagram = harness.get_diagram(ctx).await;
    for component in diagram.components() {
        let maybe_data_raw = component.resource().data.clone().expect("data is empty");
        let data = serde_json::to_string(&maybe_data_raw).expect("could not deserialize data");
        assert_eq!(
            "\"poop\"", // expected
            &data       // actual
        );
    }

    // Now, delete all the components and come back. The order at which we delete should have no
    // effect on the order of the confirmations and recommendations that come back.
    harness.create_change_set_and_update_ctx(ctx, "poop4").await;
    harness.delete_component(ctx, leo.component_id).await;
    harness.delete_component(ctx, boaster.component_id).await;
    harness.delete_component(ctx, chronicle.component_id).await;
    harness.delete_component(ctx, derke.component_id).await;
    harness.delete_component(ctx, alfajer.component_id).await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(ctx)
        .await;

    // Once the change set is applied, check the confirmations. We should now have destroy
    // recommendations.
    let confirmations = harness.list_confirmations(ctx).await;
    assert_eq!(
        10,                  // expected
        confirmations.len()  // actual
    );
    let parsed = ParsedConfirmations::new(confirmations);

    let expected = vec![
        RecommendationMetadata {
            component_id: leo.component_id,
            action_kind: ActionKind::Destroy,
        },
        RecommendationMetadata {
            component_id: chronicle.component_id,
            action_kind: ActionKind::Destroy,
        },
        RecommendationMetadata {
            component_id: boaster.component_id,
            action_kind: ActionKind::Destroy,
        },
        RecommendationMetadata {
            component_id: alfajer.component_id,
            action_kind: ActionKind::Destroy,
        },
        RecommendationMetadata {
            component_id: derke.component_id,
            action_kind: ActionKind::Destroy,
        },
    ];
    assert_eq!(
        expected,                            // expected
        parsed.recommendation_metadata_list  // actual
    );

    let mut seen_component_ids = HashSet::new();
    let mut assembler = FixRunRequestAssembler::new();
    for ((component_id, approximate_confirmation_kind), parsed_single) in parsed.inner {
        seen_component_ids.insert(component_id);

        match approximate_confirmation_kind {
            ApproximateConfirmationKind::NeedsDestroy => {
                assert_eq!(
                    ConfirmationStatus::Failure,       // expected
                    parsed_single.confirmation_status  // actual
                );

                let recommendation = parsed_single
                    .maybe_recommendation
                    .expect("recommendation not found");
                assembler.insert(
                    RecommendationMetadata {
                        component_id: recommendation.component_id,
                        action_kind: recommendation.action_kind,
                    },
                    (
                        recommendation.confirmation_attribute_value_id,
                        recommendation.recommended_action,
                    ),
                );
            }
            ApproximateConfirmationKind::NeedsCreate => {
                assert_eq!(
                    ConfirmationStatus::Success,       // expected
                    parsed_single.confirmation_status  // actual
                );
                assert!(parsed_single.maybe_recommendation.is_none());
            }
        }
    }
    assert_eq!(
        5,                        // expected
        seen_component_ids.len()  // actual
    );
    assert_eq!(
        5,               // expected
        assembler.len()  // actual
    );

    // Run the fixes for the corresponding confirmations. We will use the exact order of the
    // recommendations during assembly.
    let fix_requests = assembler.assemble(&parsed.recommendation_metadata_list);
    assert_eq!(
        5,                  // expected
        fix_requests.len()  // actual
    );
    let destroy_fix_batch_id = harness.run_fixes(ctx, fix_requests).await;

    // Check that the fix batch succeeded.
    let mut fix_batch_history_views = harness.list_fixes(ctx).await;
    assert_eq!(
        2,                             // expected
        fix_batch_history_views.len()  // actual
    );
    let first_view = fix_batch_history_views
        .pop()
        .expect("found empty batch history views");
    let second_view = fix_batch_history_views
        .pop()
        .expect("found empty batch history views");

    // Find the destroy view and ignore the create view.
    let mut destroy_view = None;
    if destroy_fix_batch_id == first_view.id {
        destroy_view = Some(first_view);
        assert_eq!(
            create_fix_batch_id, // expected
            second_view.id       // actual
        );
    } else if destroy_fix_batch_id == second_view.id {
        destroy_view = Some(second_view);
        assert_eq!(
            create_fix_batch_id, // expected
            first_view.id        // actual
        );
    }
    let destroy_view = destroy_view.expect("batch history view not found");

    // Ensure the destroy view succeeded.
    assert_eq!(
        destroy_fix_batch_id, // expected
        destroy_view.id,      // actual
    );
    assert_eq!(
        FixCompletionStatus::Success,                  // expected
        destroy_view.status.expect("no status found")  // actual
    );

    // Check confirmations on HEAD.
    let confirmations = harness.list_confirmations(ctx).await;
    assert!(confirmations.is_empty());

    // Go back to model, immediately merge and come back!
    harness.create_change_set_and_update_ctx(ctx, "poop5").await;
    harness
        .apply_change_set_and_update_ctx_visibility_to_head(ctx)
        .await;
    let confirmations = harness.list_confirmations(ctx).await;
    assert!(confirmations.is_empty());

    // TODO(nick): mix in creation and deletion recommendations as well as scenarios where not
    // all fixes are ran all at once.
}

/// The minimal data needed to work with a [`Recommendation`](Recommendation) for testing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct RecommendationMetadata {
    /// The [`ComponentId`](dal::Component) that the [`Recommendation`](Recommendation) belongs to.
    component_id: ComponentId,
    /// The [`ActionKind`](ActionKind) of the [`Recommendation`](Recommendation).
    action_kind: ActionKind,
}

/// The minimal data needed to work with a [confirmation](ConfirmationView) for testing.
#[derive(Debug)]
struct ParsedConfirmation {
    /// The (optional) [`Recommendation`]. We only expect one or zero for testing.
    maybe_recommendation: Option<Recommendation>,
    /// The status of the [confirmation](ConfirmationView).
    confirmation_status: ConfirmationStatus,
}

/// Based on the title of the [confirmation](ConfirmationView), indicate what
/// [`ActionKind`](ActionKind) the [`Recommendation`](Recommendation) would contain (if one was
/// returned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ApproximateConfirmationKind {
    /// Corresponds to [`ActionKind::Create`].
    NeedsCreate,
    /// Corresponds to [`ActionKind::Destroy`].
    NeedsDestroy,
}

/// Parsed [confirmations](ConfirmationView) containing minimal and essential data for forming
/// assertions and authoring the test.
#[derive(Debug)]
struct ParsedConfirmations {
    /// The "inner" map containing every [`ParsedConfirmation`].
    inner: HashMap<(ComponentId, ApproximateConfirmationKind), ParsedConfirmation>,
    /// A list of metadata objects corresponding  to the untouched order of
    /// [`Recommendations`](Recommendation) returned.
    recommendation_metadata_list: Vec<RecommendationMetadata>,
}

impl ParsedConfirmations {
    /// Parse a list of [confirmations](ConfirmationView) to assemble [`Self`].
    pub fn new(views: Vec<ConfirmationView>) -> Self {
        let mut inner = HashMap::new();
        let mut recommendation_metadata_list = Vec::new();

        for mut view in views {
            let approximate_confirmation_kind = if view.title == DELETE_CONFIRMATION_NAME {
                ApproximateConfirmationKind::NeedsDestroy
            } else if view.title == CREATE_CONFIRMATION_NAME {
                ApproximateConfirmationKind::NeedsCreate
            } else {
                panic!("could not find anticipated confirmation name to determine approximate confirmation kind (found confirmation title: {})", view.title);
            };

            // We should either have one or none. Those are the only two possible scenarios
            // for the schemas that we are using.
            assert!((view.recommendations.len() < 2));
            let maybe_recommendation = view.recommendations.pop();
            assert!(view.recommendations.is_empty());

            // Before we insert into the map, store some recommendation metadata. This should
            // reflect the exact order (topological and stable) that was sent from the route.
            if let Some(recommendation) = &maybe_recommendation {
                recommendation_metadata_list.push(RecommendationMetadata {
                    component_id: recommendation.component_id,
                    action_kind: recommendation.action_kind,
                });
            }

            inner.insert(
                (view.component_id, approximate_confirmation_kind),
                ParsedConfirmation {
                    maybe_recommendation,
                    confirmation_status: view.status,
                },
            );
        }

        Self {
            inner,
            recommendation_metadata_list,
        }
    }
}

/// Assemble an order of [`FixRunRequests`](FixRunRequest) that is based on a topologically sorted
/// and stable-y ordered set of [`Recommendations`](Recommendation).
#[derive(Debug)]
struct FixRunRequestAssembler {
    /// The cache needed to build a list of [`FixRunRequests`](FixRunRequest).
    cache: HashMap<RecommendationMetadata, (AttributeValueId, String)>,
}

impl FixRunRequestAssembler {
    /// Creates a new [`assembler`](FixRunRequestAssembler).
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Returns the number of cached ["recommendations"](RecommendationMetadata) that will become
    /// [`FixRunRequests`](FixRunRequest).
    pub fn len(&self) -> usize {
        self.cache.keys().len()
    }

    /// Insert a ["recommendation"](RecommendationMetadata) into the cache.
    pub fn insert(&mut self, key: RecommendationMetadata, value: (AttributeValueId, String)) {
        self.cache.insert(key, value);
    }

    /// With a given _sorted and ordered_ list of ["recommendations"](RecommendationMetadata), pull
    /// from the cache to assemble [`FixRunRequests`](FixRunRequest).
    pub fn assemble(&self, metadata_list: &Vec<RecommendationMetadata>) -> Vec<FixRunRequest> {
        let mut fix_requests: Vec<FixRunRequest> = Vec::new();
        for metadata in metadata_list {
            let (attribute_value_id, action_name) = self
                .cache
                .get(metadata)
                .expect("could not find recommendation metadata from cache");

            fix_requests.push(FixRunRequest {
                attribute_value_id: *attribute_value_id,
                component_id: metadata.component_id,
                action_name: action_name.to_owned(),
            });
        }
        fix_requests
    }
}
