use dal::DalContext;
use dal_test::{
    Result,
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_db::{
    PolicyReport,
    PolicyReportResult,
};

#[test]
async fn end_to_end(ctx: &mut DalContext) -> Result<()> {
    // Create three reports to use throughout the test.
    let divinity_report_name = "divinity original sin".to_string();
    let divinity_report_policy = "swen vincke".to_string();
    let divinity_report_report = "2014".to_string();
    PolicyReport::new_pass(
        ctx,
        divinity_report_name.to_owned(),
        divinity_report_policy.to_owned(),
        divinity_report_report.to_owned(),
    )
    .await?;

    let pillars_report_name = "pillars of eternity".to_string();
    let pillars_report_policy = "josh sawyer".to_string();
    let pillars_report_report = "2015".to_string();
    PolicyReport::new_pass(
        ctx,
        pillars_report_name.to_owned(),
        pillars_report_policy.to_owned(),
        pillars_report_report.to_owned(),
    )
    .await?;

    let fail_report_name = "fail".to_string();
    let fail_report_policy = "fail".to_string();
    let fail_report_report = "fail".to_string();
    PolicyReport::new_fail(
        ctx,
        fail_report_name.to_owned(),
        fail_report_policy.to_owned(),
        fail_report_report.to_owned(),
    )
    .await?;

    // Test that the limits work.
    {
        let reports = PolicyReport::list_with_limit(ctx, 1).await?;
        assert_eq!(
            1,             // expected
            reports.len()  // actual
        );
    }

    // Test that the reports look as we expect.
    {
        let mut reports = PolicyReport::list(ctx).await?;
        assert_eq!(
            3,             // expected
            reports.len()  // actual
        );
        let expected = vec![
            (
                divinity_report_name,
                divinity_report_policy,
                divinity_report_report,
                PolicyReportResult::Pass,
            ),
            (
                fail_report_name.to_owned(),
                fail_report_policy.to_owned(),
                fail_report_report.to_owned(),
                PolicyReportResult::Fail,
            ),
            (
                pillars_report_name,
                pillars_report_policy,
                pillars_report_report,
                PolicyReportResult::Pass,
            ),
        ];

        // Sort the reports by name for the assertion.
        let mut actual = Vec::with_capacity(reports.len());
        reports.sort_by(|a, b| a.name.cmp(&b.name));
        for report in reports {
            actual.push((report.name, report.policy, report.report, report.result));
        }

        assert_eq!(expected, actual);
    }

    Ok(())
}
