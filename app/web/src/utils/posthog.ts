import posthog from "posthog-js";

if (import.meta.env.VITE_POSTHOG_PUBLIC_KEY) {
  posthog.init(import.meta.env.VITE_POSTHOG_PUBLIC_KEY, {
    api_host: import.meta.env.VITE_POSTHOG_API_HOST,
  });
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if (window) (window as any).posthog = posthog;
}

export { posthog };
