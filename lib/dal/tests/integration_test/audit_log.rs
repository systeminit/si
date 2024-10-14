use std::collections::HashSet;

use dal::DalContext;
use dal_test::test;

#[test]
async fn generation_filtering_pagination(ctx: &DalContext) {
    let audit_logs = dal::audit_log::generate(ctx, 200)
        .await
        .expect("could not generate audit logs");
    let filtered_and_paginated_audit_logs = dal::audit_log::filter_and_paginate(
        audit_logs,
        Some(2),
        Some(25),
        None,
        None,
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
        HashSet::new(),
    )
    .expect("could not filter and paginate");
    assert_eq!(
        25,                                      // expected
        filtered_and_paginated_audit_logs.len()  // actual
    )
}
