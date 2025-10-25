use dal::DalContext;
use dal_test::{
    Result,
    helpers::{
        attribute::value,
        component,
        schema::variant,
    },
    test,
};

#[test]
async fn cycle_check_guard_test(ctx: &DalContext) -> Result<()> {
    // Cycle check should be disabled by default
    let snap = ctx.workspace_snapshot()?;
    let root = snap.root().await?;
    assert!(!snap.cycle_check().await);

    // Shouldn't be able to create a cycle when the guard is on
    {
        let _guard = snap.enable_cycle_check().await;
        assert!(snap.cycle_check().await);
        assert!(
            snap.add_edge(
                root,
                dal::EdgeWeight::new(dal::EdgeWeightKind::new_use_default()),
                root
            )
            .await
            .is_err()
        );
    }

    // Now we should be able to create the cycle
    assert!(!snap.cycle_check().await);
    snap.add_edge(
        root,
        dal::EdgeWeight::new(dal::EdgeWeightKind::new_use_default()),
        root,
    )
    .await?;

    Ok(())
}

#[test]
async fn cycle_cannot_be_created_when_cycle_check_is_enabled(ctx: &DalContext) -> Result<()> {
    variant::create(ctx, "test", TEST_ASSET_FUNCTION).await?;
    component::create(ctx, "test", "test").await?;

    // Try to create a second cycle (self-subscription) while cycle check is enabled
    {
        let _guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        assert!(
            value::subscribe(ctx, ("test", "/domain/A"), ("test", "/domain/B"))
                .await
                .is_err()
        );
    }

    Ok(())
}

#[test]
async fn cycle_can_be_created_when_cycle_check_is_disabled(ctx: &DalContext) -> Result<()> {
    variant::create(ctx, "test", TEST_ASSET_FUNCTION).await?;
    component::create(ctx, "test", "test").await?;
    component::create(ctx, "test", "test2").await?;

    // Create a cycle (self-subscription) while cycle check is disabled
    value::subscribe(ctx, ("test", "/domain/A"), ("test", "/domain/B")).await?;

    // Try to create a second cycle (self-subscription) while cycle check is enabled
    {
        let _guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        assert!(
            value::subscribe(ctx, ("test2", "/domain/A"), ("test2", "/domain/B"))
                .await
                .is_err()
        );
    }

    // Create a second cycle (self-subscription) while cycle check is disabled
    value::subscribe(ctx, ("test2", "/domain/A"), ("test2", "/domain/B")).await?;

    Ok(())
}

#[test]
async fn edges_can_be_added_when_there_is_an_existing_cycle(ctx: &DalContext) -> Result<()> {
    variant::create(ctx, "test", TEST_ASSET_FUNCTION).await?;
    component::create(ctx, "test", "test").await?;
    component::create(ctx, "test", "test2").await?;

    // Create a cycle (self-subscription) while cycle check is disabled
    value::subscribe(ctx, ("test", "/domain/A"), ("test", "/domain/B")).await?;

    // Create a non-cycle subscription while cycle check is enabled
    {
        let _guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        value::subscribe(ctx, ("test", "/domain/A"), ("test2", "/domain/B")).await?;
    }

    Ok(())
}

#[test]
async fn cycle_cannot_be_created_when_there_is_an_existing_cycle(ctx: &DalContext) -> Result<()> {
    variant::create(ctx, "test", TEST_ASSET_FUNCTION).await?;
    component::create(ctx, "test", "test").await?;
    component::create(ctx, "test", "test2").await?;

    // Create a cycle (self-subscription) while cycle check is disabled
    value::subscribe(ctx, ("test", "/domain/A"), ("test", "/domain/B")).await?;

    // Create a another cycle (self-subscription) on another component while cycle check is enabled
    {
        let _guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
        assert!(
            value::subscribe(ctx, ("test2", "/domain/A"), ("test2", "/domain/B"))
                .await
                .is_err()
        );
    }

    Ok(())
}

const TEST_ASSET_FUNCTION: &str = r#"
    function main() {
        return {
            props: [
                { name: "A", kind: "string" },
                { name: "B", kind: "string" },
            ]
        };
    }
"#;
