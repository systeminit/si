use std::collections::HashMap;

use dal::change_set::approval::ChangeSetApproval;
use dal::ComponentType;
use dal::DalContext;
use dal::HistoryActor;
use dal::SchemaVariant;
use dal_test::eyre;
use dal_test::helpers::create_component_for_default_schema_name_in_default_view;
use dal_test::helpers::create_schema;
use dal_test::prelude::ChangeSetTestHelpers;
use dal_test::sdf_test;
use dal_test::Result;
use indoc::indoc;
use permissions::ObjectType;
use permissions::Relation;
use permissions::RelationBuilder;
use pretty_assertions_sorted::assert_eq;
use sdf_server::dal_wrapper;
use sdf_server::dal_wrapper::ChangeSetApprovalCalculator;
use si_data_spicedb::SpiceDbClient;
use si_events::workspace_snapshot::EntityKind;
use si_events::ChangeSetApprovalStatus;
use si_id::ChangeSetApprovalId;

// FIXME(nick,jacob): this must happen in the "sdf_test"'s equivalent to global setup, but not in
// dal tests. This also should _really_ reflect the "schema.zed" file that production uses.
async fn write_schema(client: &mut SpiceDbClient) -> Result<()> {
    let schema = indoc! {"
        definition user {}

        definition workspace {
          relation approver: user
          relation owner: user
          permission approve = approver+owner
          permission manage = owner
        }
    "};
    client.write_schema(schema).await?;
    Ok(())
}

// NOTE(nick): this is an integration test and not a service test, but given that "sdf_test" is in
// a weird, unused place at the time of writing, this test will live here.
#[sdf_test]
async fn single_user_relation_existence_and_checksum_validility_permutations(
    ctx: &mut DalContext,
    spicedb_client: SpiceDbClient,
) -> Result<()> {
    let mut spicedb_client = spicedb_client;

    // FIXME(nick,jacob): see the comment attached to this function.
    write_schema(&mut spicedb_client).await?;

    let workspace_id = ctx.workspace_pk()?;
    let user_id = match ctx.history_actor() {
        HistoryActor::SystemInit => return Err(eyre!("invalid user")),
        HistoryActor::User(user_id) => *user_id,
    };

    // Cache hardcoded values. This should eventually become dynamic as the feature evolves.
    let entity_kind = EntityKind::SchemaVariant;
    let required_count = 1;
    let lookup_group_key = format!("workspace#{workspace_id}#approve");
    let approving_groups_without_relation =
        HashMap::from_iter(vec![(lookup_group_key.to_owned(), Vec::new())]);
    let approving_groups_with_relation =
        HashMap::from_iter(vec![(lookup_group_key, vec![user_id.to_string()])]);

    // Scenario 1: we start without any relations in SpiceDB. First, create a component, create a
    // schema variant and then approve. Second, create a second component and then approve. Both of
    // these approvals should not be applicable for fulfilling the requirements (which, as of the
    // time of writing, is only for the schema variant). The newest checksum will be valid, but it
    // will not be applicable.
    let (entity_id, first_approval_id, second_approval_id) = {
        create_component_for_default_schema_name_in_default_view(
            ctx,
            "fallout",
            "no one uses sdf test",
        )
        .await?;
        let schema = create_schema(ctx).await?;
        let (variant, _) = SchemaVariant::new(
            ctx,
            schema.id(),
            "ringo starr",
            "ringo".to_string(),
            "beatles",
            "#FFFFFF",
            ComponentType::Component,
            None,
            None,
            None,
            false,
        )
        .await?;
        let entity_id: si_id::EntityId = {
            let raw_id: si_id::ulid::Ulid = variant.id().into();
            raw_id.into()
        };
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let approving_ids = dal_wrapper::determine_approving_ids(ctx, &mut spicedb_client).await?;
        let first_approval =
            ChangeSetApproval::new(ctx, ChangeSetApprovalStatus::Approved, approving_ids).await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        create_component_for_default_schema_name_in_default_view(
            ctx,
            "fallout",
            "no like... realy it's just us",
        )
        .await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let approving_ids = dal_wrapper::determine_approving_ids(ctx, &mut spicedb_client).await?;
        let second_approval =
            ChangeSetApproval::new(ctx, ChangeSetApprovalStatus::Approved, approving_ids).await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let calculator = ChangeSetApprovalCalculator::new(ctx, &mut spicedb_client).await?;

        // TODO(nick): do not map the approvals and instead create an "expected" array for them.
        let frontend_latest_approvals = calculator.frontend_latest_approvals();
        assert_eq!(
            2,                               // expected
            frontend_latest_approvals.len()  // actual
        );
        let frontend_latest_approvals_map: HashMap<ChangeSetApprovalId, bool> =
            HashMap::from_iter(frontend_latest_approvals.iter().map(|a| (a.id, a.is_valid)));
        let expected_approvals = HashMap::from_iter(vec![
            (first_approval.id(), false),
            (second_approval.id(), true),
        ]);
        assert_eq!(
            expected_approvals,            // expected
            frontend_latest_approvals_map  // actual
        );

        let frontend_requirements = calculator
            .frontend_requirements(ctx, &mut spicedb_client)
            .await?;
        let expected_requirements = vec![si_frontend_types::ChangeSetApprovalRequirement {
            entity_id,
            entity_kind,
            required_count,
            is_satisfied: false,
            applicable_approval_ids: vec![],
            approving_groups: approving_groups_without_relation.to_owned(),
        }];
        assert_eq!(
            expected_requirements, // expected
            frontend_requirements  // actual
        );

        (entity_id, first_approval.id(), second_approval.id())
    };

    // Scenario 2: create the relation and do not create another approval. Both approvals should be
    // invalid because their checksums were based on the root node.
    let relation = {
        let relation = RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_id)
            .relation(Relation::Approver)
            .subject(ObjectType::User, user_id);
        relation.create(&mut spicedb_client).await?;

        let calculator = ChangeSetApprovalCalculator::new(ctx, &mut spicedb_client).await?;

        // TODO(nick): do not map the approvals and instead create an "expected" array for them.
        let frontend_latest_approvals = calculator.frontend_latest_approvals();
        assert_eq!(
            2,                               // expected
            frontend_latest_approvals.len()  // actual
        );
        let frontend_latest_approvals_map: HashMap<ChangeSetApprovalId, bool> =
            HashMap::from_iter(frontend_latest_approvals.iter().map(|a| (a.id, a.is_valid)));
        let expected_approvals = HashMap::from_iter(vec![
            (first_approval_id, false),
            (second_approval_id, false),
        ]);
        assert_eq!(
            expected_approvals,            // expected
            frontend_latest_approvals_map  // actual
        );

        let mut frontend_requirements = calculator
            .frontend_requirements(ctx, &mut spicedb_client)
            .await?;
        frontend_requirements
            .iter_mut()
            .for_each(|r| r.applicable_approval_ids.sort());
        let expected_requirements = vec![si_frontend_types::ChangeSetApprovalRequirement {
            entity_id,
            entity_kind,
            required_count,
            is_satisfied: false,
            applicable_approval_ids: vec![first_approval_id, second_approval_id],
            approving_groups: approving_groups_with_relation.to_owned(),
        }];
        assert_eq!(
            expected_requirements, // expected
            frontend_requirements  // actual
        );

        relation
    };

    // Scenario 3: create an approval with our relation intact. Our new approval should satisfy the
    // requirements.
    let third_approval_id = {
        let approving_ids = dal_wrapper::determine_approving_ids(ctx, &mut spicedb_client).await?;
        let third_approval =
            ChangeSetApproval::new(ctx, ChangeSetApprovalStatus::Approved, approving_ids).await?;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

        let calculator = ChangeSetApprovalCalculator::new(ctx, &mut spicedb_client).await?;

        // TODO(nick): do not map the approvals and instead create an "expected" array for them.
        let frontend_latest_approvals = calculator.frontend_latest_approvals();
        assert_eq!(
            3,                               // expected
            frontend_latest_approvals.len()  // actual
        );
        let frontend_latest_approvals_map: HashMap<ChangeSetApprovalId, bool> =
            HashMap::from_iter(frontend_latest_approvals.iter().map(|a| (a.id, a.is_valid)));
        let expected_approvals = HashMap::from_iter(vec![
            (first_approval_id, false),
            (second_approval_id, false),
            (third_approval.id(), true),
        ]);
        assert_eq!(
            expected_approvals,            // expected
            frontend_latest_approvals_map  // actual
        );

        let mut frontend_requirements = calculator
            .frontend_requirements(ctx, &mut spicedb_client)
            .await?;
        frontend_requirements
            .iter_mut()
            .for_each(|r| r.applicable_approval_ids.sort());
        let expected_requirements = vec![si_frontend_types::ChangeSetApprovalRequirement {
            entity_id,
            entity_kind,
            required_count,
            is_satisfied: true,
            applicable_approval_ids: vec![
                first_approval_id,
                second_approval_id,
                third_approval.id(),
            ],
            approving_groups: approving_groups_with_relation.to_owned(),
        }];
        assert_eq!(
            expected_requirements, // expected
            frontend_requirements  // actual
        );

        third_approval.id()
    };

    // Scenario 4: delete the relation and do not create an approval. The newest approval for the
    // original permissions should still have a valid checksum, but it should not be applicable for
    // the requirements.
    {
        relation.delete(&mut spicedb_client).await?;

        let calculator = ChangeSetApprovalCalculator::new(ctx, &mut spicedb_client).await?;

        // TODO(nick): do not map the approvals and instead create an "expected" array for them.
        let frontend_latest_approvals = calculator.frontend_latest_approvals();
        assert_eq!(
            3,                               // expected
            frontend_latest_approvals.len()  // actual
        );
        let frontend_latest_approvals_map: HashMap<ChangeSetApprovalId, bool> =
            HashMap::from_iter(frontend_latest_approvals.iter().map(|a| (a.id, a.is_valid)));
        let expected_approvals = HashMap::from_iter(vec![
            (first_approval_id, false),
            (second_approval_id, true),
            (third_approval_id, false),
        ]);
        assert_eq!(
            expected_approvals,            // expected
            frontend_latest_approvals_map  // actual
        );

        let frontend_requirements = calculator
            .frontend_requirements(ctx, &mut spicedb_client)
            .await?;
        let expected_requirements = vec![si_frontend_types::ChangeSetApprovalRequirement {
            entity_id,
            entity_kind,
            required_count,
            is_satisfied: false,
            applicable_approval_ids: vec![],
            approving_groups: approving_groups_without_relation.to_owned(),
        }];
        assert_eq!(
            expected_requirements, // expected
            frontend_requirements  // actual
        );
    }

    // Scenario 5: re-create the relation, but do not create an approval. The newest approval
    // should still have a valid checksum and it should satisfy the requirements, once again.
    {
        relation.create(&mut spicedb_client).await?;

        let calculator = ChangeSetApprovalCalculator::new(ctx, &mut spicedb_client).await?;

        // TODO(nick): do not map the approvals and instead create an "expected" array for them.
        let frontend_latest_approvals = calculator.frontend_latest_approvals();
        assert_eq!(
            3,                               // expected
            frontend_latest_approvals.len()  // actual
        );
        let frontend_latest_approvals_map: HashMap<ChangeSetApprovalId, bool> =
            HashMap::from_iter(frontend_latest_approvals.iter().map(|a| (a.id, a.is_valid)));
        let expected_approvals = HashMap::from_iter(vec![
            (first_approval_id, false),
            (second_approval_id, false),
            (third_approval_id, true),
        ]);
        assert_eq!(
            expected_approvals,            // expected
            frontend_latest_approvals_map  // actual
        );

        let mut frontend_requirements = calculator
            .frontend_requirements(ctx, &mut spicedb_client)
            .await?;
        frontend_requirements
            .iter_mut()
            .for_each(|r| r.applicable_approval_ids.sort());
        let expected_requirements = vec![si_frontend_types::ChangeSetApprovalRequirement {
            entity_id,
            entity_kind,
            required_count,
            is_satisfied: true,
            applicable_approval_ids: vec![first_approval_id, second_approval_id, third_approval_id],
            approving_groups: approving_groups_with_relation,
        }];
        assert_eq!(
            expected_requirements, // expected
            frontend_requirements  // actual
        );
    }

    Ok(())
}
