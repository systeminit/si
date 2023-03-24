import { PostHog } from 'posthog-node';
import { asyncExitHook } from 'exit-hook';

export const posthog = new PostHog(
  process.env.POSTHOG_PUBLIC_KEY as string,
  { host: process.env.POSTHOG_API_HOST as string },
);

asyncExitHook(async () => {
  await posthog.shutdownAsync();
}, { minimumWait: 500 });
