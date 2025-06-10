use std::time::Duration;

use dal::{
    Component,
    DalContext,
    qualification::{
        QualificationOutputStreamView,
        QualificationResult,
        QualificationSubCheck,
        QualificationSubCheckStatus,
        QualificationView,
    },
};
use dal_test::{
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_default_schema_name_in_default_view,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

const DUMMY_OUTPUT_STREAM_VIEW_LINE: &str = "[THIS IS FAKE DATA]";

#[test]
async fn list_qualifications(ctx: &mut DalContext) {
    let component = create_component_for_default_schema_name_in_default_view(
        ctx,
        "dummy-secret",
        "deserializing serde json value null into an option results in None and that's insane",
    )
    .await
    .expect("could not create component");

    // Prepare expected qualification views.
    let expected_additional_qualification_view_name =
        "test:qualificationDummySecretStringIsTodd".to_string();
    let expected_additional_qualification_view = QualificationView {
        title: expected_additional_qualification_view_name.to_owned(),
        output: vec![QualificationOutputStreamView {
            stream: "output".to_string(),
            line: DUMMY_OUTPUT_STREAM_VIEW_LINE.to_string(),
            level: "info".to_string(),
        }],
        finalized: true,
        description: None,
        link: None,
        result: Some(QualificationResult {
            status: QualificationSubCheckStatus::Failure,
            title: Some(expected_additional_qualification_view_name.to_owned()),
            link: None,
            sub_checks: vec![QualificationSubCheck {
                description: "dummy secret string is empty".to_string(),
                status: QualificationSubCheckStatus::Failure,
            }],
        }),
        qualification_name: expected_additional_qualification_view_name.to_owned(),
    };

    // Check qualifications before committing and running dependent values update. We do not need
    // to sanitize the views here as there should be no output.
    let qualifications = Component::list_qualifications(ctx, component.id())
        .await
        .expect("could not list qualifications");
    assert!(qualifications.is_empty());

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

    // Sanitize the qualification views' output stream view lines.
    let qualifications: Vec<QualificationView> = qualifications
        .into_iter()
        .map(replace_output_stream_view_line_contents)
        .collect();

    // NOTE(nick): at the time of writing this test, we receive the qualifications sorted, so we
    // neither need to perform an additional sort nor use something like a hash set.
    assert_eq!(
        vec![replace_output_stream_view_line_contents(
            expected_additional_qualification_view
        )], // expected
        qualifications // actual
    );
}

fn replace_output_stream_view_line_contents(view: QualificationView) -> QualificationView {
    let mut view = view;
    for output_stream_view in &mut view.output {
        output_stream_view.line = DUMMY_OUTPUT_STREAM_VIEW_LINE.to_string();
    }
    view
}
