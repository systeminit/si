use dal::input_sources::InputSources;
use dal::DalContext;
use dal_test::test;

#[test]
async fn assembling_input_sources_works(ctx: &mut DalContext) {
    let _input_sources = InputSources::assemble_for_all_schema_variants(ctx)
        .await
        .expect("could not assemble input sources");
}
