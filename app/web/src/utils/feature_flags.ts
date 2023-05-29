import { posthog } from "@/utils/posthog";

export const SINGLE_MODEL_SCREEN_FF = posthog.isFeatureEnabled(
  "one_screen_to_rule_them_all",
);
