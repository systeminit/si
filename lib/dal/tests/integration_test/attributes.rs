use dal::{
    DalContext,
    attribute::attributes,
};
use dal_test::{
    Result,
    helpers::{
        attribute::value::{
            self,
        },
        component::{
            self,
        },
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

// Test that updating attributes sets them (and their parents) correctly, but leaves default
// values and other values alone.
#[test]
async fn update_attributes(ctx: &DalContext) -> Result<()> {
    variant::create(
        ctx,
        "test",
        r#"
            function main() {
                return {
                    props: [
                        { name: "Parent", kind: "object", children: [
                            { name: "Updated", kind: "string" },
                            { name: "New", kind: "string" },
                            { name: "Unchanged", kind: "string" },
                            { name: "Missing", kind: "string" },
                            { name: "Default", kind: "string", defaultValue: "default" },
                        ]},
                        { name: "Missing", kind: "string" },
                        { name: "Default", kind: "string", defaultValue: "default" },
                    ]
                };
            }
        "#,
    )
    .await?;

    // Set some initial values and make sure they are set correctly without messing with other
    // values or defaults.
    let component_id = component::create(ctx, "test", "test").await?;
    value::set(ctx, ("test", "/domain/Parent/Updated"), "old").await?;
    value::set(ctx, ("test", "/domain/Parent/Unchanged"), "old").await?;
    assert_eq!(
        json!({
            "Parent": {
                "Updated": "old",
                "Unchanged": "old",
                "Default": "default",
            },
            "Default": "default",
        }),
        component::domain(ctx, "test").await?
    );
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Updated")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/New")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Unchanged")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Missing")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Default")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Default")).await?);

    // Update values and make sure they are updated correctly without messing with other values
    // or defaults.
    attributes::update_attributes(
        ctx,
        component_id,
        serde_json::from_value(json!({
            "/domain/Parent/Updated": "new",
            "/domain/Parent/New": "new",
        }))?,
    )
    .await?;
    assert_eq!(
        json!({
            "Parent": {
                "Updated": "new",
                "New": "new",
                "Unchanged": "old",
                "Default": "default",
            },
            "Default": "default",
        }),
        component::domain(ctx, "test").await?
    );
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Updated")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/New")).await?);
    assert!(value::is_set(ctx, ("test", "/domain/Parent/Unchanged")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Missing")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Parent/Default")).await?);
    assert!(!value::is_set(ctx, ("test", "/domain/Default")).await?);

    Ok(())
}
