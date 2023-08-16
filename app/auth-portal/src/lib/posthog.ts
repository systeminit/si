import posthog, { CaptureOptions, Properties } from "posthog-js";

if (!import.meta.env.SSR && import.meta.env.VITE_POSTHOG_PUBLIC_KEY) {
  posthog.init(import.meta.env.VITE_POSTHOG_PUBLIC_KEY, {
    api_host: import.meta.env.VITE_POSTHOG_API_HOST,
  });

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  if (window) (window as any).posthog = posthog;
}

// small wrapper makes it easier to swap things later,
// calls multiple tracking platforms, or transform data for different platforms
export const tracker = {
  identify: posthog.identify,
  alias: posthog.alias,
  trackEvent(
    eventName: string,
    properties?: Properties,
    options?: CaptureOptions,
  ) {
    if (import.meta.env.VITE_DISABLE_TRACKING) {
      // eslint-disable-next-line no-console
      console.log("TRACK EVENT", `ap-${eventName}`, properties, options);
      return;
    }

    // add consistent prefix for all events coming from this part of the stack
    posthog.capture(`ap-${eventName}`, properties, options);
  },
};
