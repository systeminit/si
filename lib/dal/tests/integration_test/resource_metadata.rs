use dal::{
    DalContext,
    Secret,
    Workspace,
    resource_metadata,
};
use dal_test::{
    Result,
    WorkspaceSignup,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        change_set,
        component,
        encrypt_message,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_events::{
    ResourceMetadata,
    ResourceStatus,
};

#[test]
async fn list(ctx: &mut DalContext, nw: &WorkspaceSignup) -> Result<()> {
    component::create(ctx, "dummy-secret", "source").await?;
    component::create(ctx, "fallout", "destination").await?;
    change_set::commit(ctx).await?;

    // Cache the name of the secret definition from the test exclusive schema. Afterward, cache the
    // prop we need for attribute value update.
    value::subscribe(
        ctx,
        ("destination", "/secrets/dummy"),
        ("source", "/secrets/dummy"),
    )
    .await?;
    change_set::commit(ctx).await?;

    // Create the secret and commit.
    let secret = Secret::new(
        ctx,
        "toto wolff",
        "dummy".to_string(),
        None,
        &encrypt_message(ctx, nw.key_pair.pk(), &serde_json::json![{"value": "todd"}])
            .await
            .expect("could not encrypt message"),
        nw.key_pair.pk(),
        Default::default(),
        Default::default(),
    )
    .await?;
    change_set::commit(ctx).await?;

    // Use the secret in the source component and commit.
    Secret::attach_for_attribute_value(
        ctx,
        value::id(ctx, ("source", "/secrets/dummy")).await?,
        Some(secret.id()),
    )
    .await?;
    change_set::commit(ctx).await?;

    // Set the workspace token to mimic how it works with the auth-api.
    Workspace::get_by_pk(ctx, ctx.tenancy().workspace_pk().expect("workspace"))
        .await?
        .set_token(ctx, "token".to_string())
        .await?;

    // Ensure that the parent is head so that the "create" action will execute by default.
    // Technically, this primarily validates the test setup rather than the system itself, but it
    // serves a secondary function of ensuring no prior functions cause this assertion to fail.
    assert!(ctx.parent_is_head().await?);

    // Apply to the base change set and commit.
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set");

    // Wait for all actions to run.
    ChangeSetTestHelpers::wait_for_actions_to_run(ctx)
        .await
        .expect("deadline for actions to run exceeded");

    // Validate that both components look as expected on HEAD.
    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "source",
                "type": "component",
            },
            "secrets": {
                "dummy": secret.encrypted_secret_key().to_string()
            },
            "resource_value": {},
            "qualification": {
                "test:qualificationDummySecretStringIsTodd": {
                    "result": "success",
                    "message": "dummy secret string matches expected value"
                },
            },
        }], // expected
        component::value(ctx, "source").await?
    );
    let last_synced = value::get(ctx, ("destination", "/resource/last_synced")).await?;
    assert_eq!(
        serde_json::json![{
            "si": {
                "color": "#ffffff",
                "name": "destination",
                "type": "component",
            },
            "domain": {
                "name": "destination",
                "active": true
            },
            "secrets": {
                "dummy": secret.encrypted_secret_key().to_string()
            },
            "resource": {
                "status": "ok",
                "payload": { "poop" :true },
                "last_synced": last_synced.clone(),
            },
            "resource_value": {}
        }], // expected
        component::value(ctx, "destination").await?
    );

    // Finally, we can collect the resource metadata.
    let metadata = resource_metadata::list(ctx).await?;
    let expected = ResourceMetadata {
        component_id: component::id(ctx, "destination").await?,
        status: ResourceStatus::Ok,
        last_synced: serde_json::from_value(last_synced)?,
    };
    assert_eq!(
        vec![expected], // expected
        metadata,       // actual
    );

    Ok(())
}
