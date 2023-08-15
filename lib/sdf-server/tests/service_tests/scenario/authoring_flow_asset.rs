use axum::Router;
use dal_test::{sdf_test, AuthToken, DalContextHead};
use pretty_assertions_sorted::assert_eq;

use crate::service_tests::scenario::ScenarioHarness;

/// This test runs through the entire authoring flow for a new asset (not the funcs associated)
#[sdf_test]
#[ignore]
async fn authoring_flow_asset(
    DalContextHead(mut ctx): DalContextHead,
    app: Router,
    AuthToken(auth_token): AuthToken,
) {
    // Setup the harness to start.
    let mut harness = ScenarioHarness::new(&ctx, app, auth_token, &[]).await;

    let schema_name = "si-demo-schema";

    // Enter a new change set. We will not go through the routes for this.
    harness
        .create_change_set_and_update_ctx(&mut ctx, ScenarioHarness::generate_fake_name())
        .await;

    // Create the asset
    let asset = harness
        .create_asset(ctx.visibility(), schema_name.to_string(), None)
        .await;
    assert!(asset.asset_id.is_some());

    // Update the asset with the schema
    // harness.update_asset(&ctx, )
    let asset_definition = r#"function createAsset() {
        const imageProp = new PropBuilder()
            .setKind("string")
            .setName("image")
            .setValueFrom(new ValueFromBuilder().setKind("prop").setPropPath(["root", "si", "name"]).build())
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build();

        const portsProp = new PropBuilder()
            .setKind("array")
            .setName("ExposedPorts")
            .setEntry(new PropBuilder()
                        .setName("ExposedPort")
                        .setKind("string")
                        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
                        .build())
            .build();

        const portsSocket = new SocketDefinitionBuilder()
            .setName("Exposed Ports")
            .setArity("many")
            .build();

        const credentialSocket = new SocketDefinitionBuilder()
            .setName("Docker Hub Credential")
            .setArity("many")
            .build();


        return new AssetBuilder()
            .addProp(imageProp)
            .addProp(portsProp)
            .addInputSocket(credentialSocket)
            .addOutputSocket(portsSocket)
            .build();
    }"#;

    harness
        .update_asset(
            ctx.visibility(),
            asset.asset_id,
            schema_name.to_string(),
            None,
            asset_definition.to_string(),
        )
        .await;

    harness
        .publish_asset(
            ctx.visibility(),
            asset.asset_id,
            schema_name.to_string(),
            None,
            asset_definition.to_string(),
        )
        .await;

    // Let's add the new schema to our test harness cache
    harness.add_schemas(&ctx, &[schema_name]).await;

    let my_asset = harness
        .create_node(ctx.visibility(), schema_name, None)
        .await;

    // Update the name of the asset
    harness
        .update_value(
            &ctx,
            my_asset.component_id,
            &["si", "name"],
            Some(serde_json::json!("systeminit/whiskers")),
        )
        .await;

    // Ensure that the dependent value updates have propagated
    assert_eq!(
        serde_json::json![{
            "si": {
                "name": "systeminit/whiskers",
                "type": "component",
                "color": "#FFFF00",
                "protected": false,
            },
            "domain": {
                "image": "systeminit/whiskers",
            },
        }], // expected
        my_asset
            .view(&ctx)
            .await
            .to_value()
            .expect("could not convert to value"), // actual
    );
}
