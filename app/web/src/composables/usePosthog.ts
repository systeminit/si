import { PostHog } from "posthog-js";
import { inject } from "vue";

export function usePosthog(): PostHog {
  return inject("posthog") as PostHog;
}
