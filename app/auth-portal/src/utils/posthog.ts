import posthog from "posthog-js";

if (import.meta.env.VITE_POSTHOG_PUBLIC_KEY) {
  posthog.init(import.meta.env.VITE_POSTHOG_PUBLIC_KEY, {
    api_host: import.meta.env.VITE_POSTHOG_API_HOST,
  });
}

export { posthog };
