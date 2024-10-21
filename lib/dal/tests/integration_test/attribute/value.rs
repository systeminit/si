use dal::{AttributeValue, DalContext};
use dal_test::expected::ExpectComponent;
use dal_test::helpers::ChangeSetTestHelpers;
use dal_test::{test, Result};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test]
async fn arguments_for_prototype_function_execution(ctx: &mut DalContext) -> Result<()> {
    // Create a component and commit. For context, the test exclusive schema has the identity
    // function set on "/root/domain/name" with an input from "/root/si/name". We need to ensure
    // that the value of "/root/si/name" comes in, as expected. The name is set when creating a
    // component, so we do not need to do additional setup.
    let expected = "you should see this name in the arguments";
    let component = ExpectComponent::create_named(ctx, "swifty", expected).await;
    let name_prop = component.prop(ctx, ["root", "domain", "name"]).await;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Ensure that the arguments look as we expect.
    let name_av_id = name_prop.attribute_value(ctx).await.id();
    let (_, arguments) =
        AttributeValue::prepare_arguments_for_prototype_function_execution(ctx, name_av_id).await?;
    assert_eq!(
        json![{
            "identity": expected
        }],
        arguments
    );
    Ok(())
}
