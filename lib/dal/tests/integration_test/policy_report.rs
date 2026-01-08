use dal::DalContext;
use dal_test::{
    Result,
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_db::{
    DEFAULT_PAGE_SIZE,
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

    // Test that asking for fewer results than the total works.
    {
        let batch = PolicyReport::fetch_batch(ctx, Some(1), Some(1)).await?;
        assert_eq!(
            1,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            3,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            1,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );
    }

    // Test that the reports look as we expect.
    {
        let batch = PolicyReport::fetch_batch(ctx, None, None).await?;
        assert_eq!(
            3,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            3,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            DEFAULT_PAGE_SIZE, // expected
            batch.page_size    // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            1,                      // expected
            batch.total_page_count  // actual
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
        let mut reports = batch.reports;
        let mut actual = Vec::with_capacity(reports.len());
        reports.sort_by(|a, b| a.name.cmp(&b.name));
        for report in reports {
            actual.push((report.name, report.policy, report.report, report.result));
        }

        assert_eq!(expected, actual);
    }

    // Test minimum page size and page number.
    {
        // Both are zero.
        let batch = PolicyReport::fetch_batch(ctx, Some(0), Some(0)).await?;
        assert_eq!(
            1,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            3,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            1,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );

        // Page size is zero.
        let batch = PolicyReport::fetch_batch(ctx, Some(0), None).await?;
        assert_eq!(
            1,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            3,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            1,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );

        // Page number is zero.
        let batch = PolicyReport::fetch_batch(ctx, None, Some(0)).await?;
        assert_eq!(
            3,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            3,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            DEFAULT_PAGE_SIZE, // expected
            batch.page_size    // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            1,                      // expected
            batch.total_page_count  // actual
        );
    }

    // Test pagination with five reports across three pages.
    {
        PolicyReport::new_fail(
            ctx,
            "fourth".to_string(),
            "fourth".to_string(),
            "fourth".to_string(),
        )
        .await?;
        PolicyReport::new_fail(
            ctx,
            "fifth".to_string(),
            "fifth".to_string(),
            "fifth".to_string(),
        )
        .await?;

        // Grab the first page.
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(1)).await?;
        assert_eq!(
            2,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            5,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            2,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );

        // Grab the second page.
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(2)).await?;
        assert_eq!(
            2,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            5,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            2,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            2,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );

        // Grab the third page.
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(3)).await?;
        assert_eq!(
            1,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            5,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            2,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            3,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );

        // What if you ask for a fourth page that does not exist?.
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(4)).await?;
        dbg!(&batch);
        assert_eq!(
            0,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            5,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            2,               // expected
            batch.page_size  // actual
        );
        assert_eq!(
            4,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            3,                      // expected
            batch.total_page_count  // actual
        );

        // No pagination, just to be sure!
        let batch = PolicyReport::fetch_batch(ctx, None, None).await?;
        assert_eq!(
            5,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            5,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(
            DEFAULT_PAGE_SIZE, // expected
            batch.page_size    // actual
        );
        assert_eq!(
            1,                 // expected
            batch.page_number  // actual
        );
        assert_eq!(
            1,                      // expected
            batch.total_page_count  // actual
        );
    }

    Ok(())
}
