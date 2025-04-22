use std::collections::HashSet;

use dal::{
    DalContext,
    prompt_override::PromptOverride,
};
use dal_test::{
    color_eyre::Result,
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn update_prompt(ctx: &mut DalContext) -> Result<()> {
    // Add prompt
    assert_eq!(PromptOverride::list(ctx).await?, HashSet::new());
    assert_eq!(PromptOverride::get_opt(ctx, "AssetSchema").await?, None);

    // Add prompt
    PromptOverride::set(ctx, "AssetSchema", "hi 1").await?;
    assert_eq!(
        PromptOverride::list(ctx).await?,
        ["AssetSchema"].into_iter().map(ToOwned::to_owned).collect()
    );
    assert_eq!(
        PromptOverride::get_opt(ctx, "AssetSchema").await?,
        Some("hi 1".to_string())
    );

    // Update prompt
    PromptOverride::set(ctx, "AssetSchema", "hi 2").await?;
    assert_eq!(
        PromptOverride::list(ctx).await?,
        ["AssetSchema"].into_iter().map(ToOwned::to_owned).collect()
    );
    assert_eq!(
        PromptOverride::get_opt(ctx, "AssetSchema").await?,
        Some("hi 2".to_string())
    );

    // Reset prompt
    PromptOverride::reset(ctx, "AssetSchema").await?;
    assert_eq!(PromptOverride::list(ctx).await?, HashSet::new());
    assert_eq!(PromptOverride::get_opt(ctx, "AssetSchema").await?, None);

    // Reset prompt that doesn't exist
    PromptOverride::reset(ctx, "AssetSchema").await?;
    assert_eq!(PromptOverride::list(ctx).await?, HashSet::new());
    assert_eq!(PromptOverride::get_opt(ctx, "AssetSchema").await?, None);

    // Add prompt back
    PromptOverride::set(ctx, "AssetSchema", "hi 3").await?;
    assert_eq!(
        PromptOverride::list(ctx).await?,
        ["AssetSchema"].into_iter().map(ToOwned::to_owned).collect()
    );
    assert_eq!(
        PromptOverride::get_opt(ctx, "AssetSchema").await?,
        Some("hi 3".to_string())
    );

    // Add another prompt
    PromptOverride::set(ctx, "CreateAction", "hi 3").await?;
    assert_eq!(
        PromptOverride::list(ctx).await?,
        ["AssetSchema", "CreateAction"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect()
    );
    assert_eq!(
        PromptOverride::get_opt(ctx, "AssetSchema").await?,
        Some("hi 3".to_string())
    );

    Ok(())
}
