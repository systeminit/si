use super::state::PosthogClient;
use dal::{DalContext, HistoryActor};
use si_posthog::FeatureFlag;

#[allow(unused)]
pub async fn feature_is_enabled(
    ctx: &DalContext,
    posthog_client: &PosthogClient,
    feature: FeatureFlag,
) -> bool {
    match ctx.history_actor() {
        HistoryActor::SystemInit => false,
        HistoryActor::User(user_pk) => posthog_client
            .check_feature_flag(feature, user_pk.to_string())
            .await
            .unwrap_or(false),
    }
}
