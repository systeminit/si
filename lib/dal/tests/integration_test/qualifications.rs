use std::time::Duration;

use dal::qualification::{
    QualificationOutputStreamView, QualificationResult, QualificationSubCheck,
    QualificationSubCheckStatus, QualificationView,
};
use dal::{Component, DalContext};
use dal_test::helpers::{create_component_for_schema_name, ChangeSetTestHelpers};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn list_qualifications(ctx: &mut DalContext) {
    let component = create_component_for_schema_name(
        ctx,
        "dummy-secret",
        "deserializing serde json value null into an option results in None and that's insane",
    )
    .await;

    // Prepare expected qualification views.
    let expected_prop_validations_qualification_view = QualificationView {
        title: "Prop Validations".to_string(),
        output: vec![],
        finalized: true,
        description: None,
        link: None,
        result: Some(QualificationResult {
            status: QualificationSubCheckStatus::Success,
            title: None,
            link: None,
            sub_checks: vec![QualificationSubCheck {
                description: "Component has 0 invalid value(s).".to_string(),
                status: QualificationSubCheckStatus::Success,
            }],
        }),
        qualification_name: "validations".to_string(),
    };
    let expected_additional_qualification_view_name =
        "test:qualificationDummySecretStringIsTodd".to_string();
    let expected_additional_qualification_view = QualificationView {
        title: expected_additional_qualification_view_name.to_owned(),
        output: vec![
            QualificationOutputStreamView {
                stream: "output".to_string(),
                line: "Output: {\n  \"protocol\": \"result\",\n  \"status\": \"success\",\n  \"executionId\": \"tomcruise\",\n  \"data\": {\n    \"result\": \"failure\",\n    \"message\": \"dummy secret string is empty\"\n  },\n  \"unset\": false\n}".to_string(),
                level: "info".to_string(),
            }
        ],
        finalized: true,
        description: None,
        link: None,
        result: Some(QualificationResult {
            status: QualificationSubCheckStatus::Failure,
            title: Some( expected_additional_qualification_view_name.to_owned()),
            link: None,
            sub_checks: vec![QualificationSubCheck {
                description: "dummy secret string is empty".to_string(),
                status: QualificationSubCheckStatus::Failure,
            }],
        }),
        qualification_name:  expected_additional_qualification_view_name.to_owned(),
    };

    // Check qualifications before committing and running dependent values update.
    let qualifications = Component::list_qualifications(ctx, component.id())
        .await
        .expect("could not list qualifications");
    assert_eq!(
        vec![expected_prop_validations_qualification_view.clone()], // expected
        qualifications                                              // actual
    );

    // Commit and check qualifications again. We should see the populated map with all
    // qualifications.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot to visibility");

    let total_count = 100;
    let mut count = 0;

    let mut qualifications;
    loop {
        if count > total_count {
            panic!("func run log entries have not all been finalized after waiting for a period");
        }

        qualifications = Component::list_qualifications(ctx, component.id())
            .await
            .expect("could not list qualifications");

        if qualifications.iter().all(|qual| qual.finalized) {
            break;
        }

        tokio::time::sleep(Duration::from_millis(100)).await;

        count += 1;
    }

    // NOTE(nick): at the time of writing this test, we receive the qualifications sorted, so we
    // neither need to perform an additional sort nor use something like a hash set.
    assert_eq!(
        vec![
            expected_prop_validations_qualification_view,
            expected_additional_qualification_view
        ], // expected
        qualifications // actual
    );
}
