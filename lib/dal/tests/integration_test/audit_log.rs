use dal::DalContext;
use dal_test::test;

#[test]
async fn audit_log_generation_works(ctx: &DalContext) {
    let _logs = dal::audit_log::generate(ctx)
        .await
        .expect("could not generate audit logs");
}
