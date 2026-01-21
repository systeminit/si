use dal::DalContext;
use dal_test::{
    Result,
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_db::{
    DEFAULT_PAGE_SIZE,
    MAX_REPORTS_PER_GROUP,
    PolicyReport,
};

#[test]
async fn end_to_end(ctx: &mut DalContext) -> Result<()> {
    let policy_name = "test-policy".to_string();
    let other_policy_name = "other-policy".to_string();

    // Create multiple reports with the same name for pagination testing
    PolicyReport::new_pass(
        ctx,
        policy_name.clone(),
        "policy document 1".to_string(),
        "report 1".to_string(),
    )
    .await?;

    PolicyReport::new_pass(
        ctx,
        policy_name.clone(),
        "policy document 2".to_string(),
        "report 2".to_string(),
    )
    .await?;

    PolicyReport::new_fail(
        ctx,
        policy_name.clone(),
        "policy document 3".to_string(),
        "report 3".to_string(),
    )
    .await?;

    // Create a report with a different name to ensure filtering works
    PolicyReport::new_pass(
        ctx,
        other_policy_name.clone(),
        "other policy document".to_string(),
        "other report".to_string(),
    )
    .await?;

    // Test that fetching by name only returns reports with that name.
    {
        let batch = PolicyReport::fetch_batch(ctx, None, None, policy_name.clone()).await?;
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

        // Verify all reports have the correct name
        for report in &batch.reports {
            assert_eq!(&policy_name, &report.name);
        }
    }

    // Test that fetching a different policy name only returns that policy
    {
        let batch = PolicyReport::fetch_batch(ctx, None, None, other_policy_name.clone()).await?;
        assert_eq!(
            1,                   // expected
            batch.reports.len()  // actual
        );
        assert_eq!(
            1,                        // expected
            batch.total_report_count  // actual
        );
        assert_eq!(&other_policy_name, &batch.reports[0].name);
    }

    // Test that asking for fewer results than the total works with pagination.
    {
        let batch = PolicyReport::fetch_batch(ctx, Some(1), Some(1), policy_name.clone()).await?;
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

    // Test minimum page size and page number.
    {
        // Both are zero.
        let batch = PolicyReport::fetch_batch(ctx, Some(0), Some(0), policy_name.clone()).await?;
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
        let batch = PolicyReport::fetch_batch(ctx, Some(0), None, policy_name.clone()).await?;
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
        let batch = PolicyReport::fetch_batch(ctx, None, Some(0), policy_name.clone()).await?;
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
            policy_name.clone(),
            "fourth".to_string(),
            "fourth".to_string(),
        )
        .await?;
        PolicyReport::new_fail(
            ctx,
            policy_name.clone(),
            "fifth".to_string(),
            "fifth".to_string(),
        )
        .await?;

        // Grab the first page.
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(1), policy_name.clone()).await?;
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
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(2), policy_name.clone()).await?;
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
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(3), policy_name.clone()).await?;
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
        let batch = PolicyReport::fetch_batch(ctx, Some(2), Some(4), policy_name.clone()).await?;
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
        let batch = PolicyReport::fetch_batch(ctx, None, None, policy_name.clone()).await?;
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

#[test]
async fn grouped_by_name(ctx: &mut DalContext) -> Result<()> {
    // Test Case 1: Create a policy with 1 report
    let policy_one_report = "policy-with-one-report".to_string();
    PolicyReport::new_pass(
        ctx,
        policy_one_report.clone(),
        "policy document 1".to_string(),
        "report 1".to_string(),
    )
    .await?;

    // Test Case 2: Create a policy with 2 reports
    let policy_two_reports = "policy-with-two-reports".to_string();
    PolicyReport::new_pass(
        ctx,
        policy_two_reports.clone(),
        "policy document 1".to_string(),
        "report 1".to_string(),
    )
    .await?;
    PolicyReport::new_fail(
        ctx,
        policy_two_reports.clone(),
        "policy document 2".to_string(),
        "report 2".to_string(),
    )
    .await?;

    // Test Case 3: Create a policy with more than MAX_REPORTS_PER_GROUP reports
    let policy_many_reports = "policy-with-many-reports".to_string();
    for i in 1..=15 {
        PolicyReport::new_pass(
            ctx,
            policy_many_reports.clone(),
            format!("policy document {i}"),
            format!("report {i}"),
        )
        .await?;
    }

    // Fetch the grouped reports
    let groups = PolicyReport::fetch_grouped_by_name(ctx).await?;

    // Verify we got 3 groups
    assert_eq!(
        3,            // expected
        groups.len()  // actual
    );

    // Sort groups by name for consistent ordering
    let mut groups = groups;
    groups.sort_by(|a, b| a.name.cmp(&b.name));

    // Verify group 1: policy with many reports (should be limited to MAX_REPORTS_PER_GROUP)
    assert_eq!(&policy_many_reports, &groups[0].name);
    assert_eq!(
        MAX_REPORTS_PER_GROUP as usize, // expected
        groups[0].results.len()         // actual
    );
    // Verify all reports have the correct name
    for report in &groups[0].results {
        assert_eq!(&policy_many_reports, &report.name);
    }

    // Verify we got exactly MAX_REPORTS_PER_GROUP reports, not all 15
    // This confirms the limit is working correctly
    assert!(
        groups[0].results.len() < 15,
        "Should have limited to {MAX_REPORTS_PER_GROUP} reports, not returned all 15"
    );

    // Note(victor): We cannot reliably test ORDER BY created_at DESC here because all reports
    // created within the same test transaction have identical created_at timestamps.
    // The SQL query is correct, but the ordering is indeterminate when timestamps are equal.

    // Verify group 2: policy with one report
    assert_eq!(&policy_one_report, &groups[1].name);
    assert_eq!(
        1,                       // expected
        groups[1].results.len()  // actual
    );
    assert_eq!(&policy_one_report, &groups[1].results[0].name);

    // Verify group 3: policy with two reports
    assert_eq!(&policy_two_reports, &groups[2].name);
    assert_eq!(
        2,                       // expected
        groups[2].results.len()  // actual
    );
    for report in &groups[2].results {
        assert_eq!(&policy_two_reports, &report.name);
    }

    Ok(())
}
