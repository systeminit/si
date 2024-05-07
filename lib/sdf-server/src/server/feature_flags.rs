use dal::DalContext;
use serde::{Deserialize, Serialize};
use si_settings::ValueKind;
use strum::Display;

#[derive(Debug, Display, Deserialize, Serialize, Clone, clap::ValueEnum, Hash, Eq, PartialEq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "snake_case")]
pub enum FeatureFlag {
    Secrets,
    ActionsV2,
}

impl From<FeatureFlag> for ValueKind {
    fn from(value: FeatureFlag) -> Self {
        ValueKind::String(value.to_string())
    }
}

// TODO create feature flags service
#[allow(unused)]
pub async fn feature_is_enabled(
    ctx: &DalContext,
    // posthog_client: &PosthogClient,
    feature: FeatureFlag,
) -> bool {
    false
    // ctx.services_context().fe

    // match ctx.history_actor() {
    //     HistoryActor::SystemInit => false,
    //     HistoryActor::User(user_pk) => posthog_client
    //         .check_feature_flag(feature.to_string(), user_pk.to_string())
    //         .await
    //         .unwrap_or(false),
    // }
}
