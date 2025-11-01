/**
 * Analytics Module - PostHog Integration
 *
 * This module provides analytics tracking functionality using PostHog for
 * telemetry and usage analytics. It handles event tracking with session
 * management and graceful degradation for anonymous usage.
 *
 * @example
 * ```ts
 * import { Analytics } from "./analytics.ts";
 *
 * const analytics = new Analytics("si-tmpl");
 *
 * // Track an event
 * analytics.trackEvent("feature-used", { feature: "generate" });
 *
 * // Cleanup on exit
 * await analytics.shutdown();
 * ```
 *
 * @module
 */

import { PostHog } from "posthog-node";

/**
 * PostHog production API key.
 *
 * NOTE: This key is publicly available as a PostHog API key and is intended to
 * be shared and used (i.e. this is not an accidentally leaked key).
 */
const POSTHOG_PROD_API_KEY = "phc_KpehlXOqtU44B2MeW6WjqR09NxRJCYEiUReA58QcAYK";

/** PostHog production host URL for event ingestion. */
const POSTHOG_PROD_HOST = "https://e.systeminit.com";

/** Default maximum time in milliseconds to wait for shutdown to complete. */
const POSTHOG_DEFAULT_SHUTDOWN_MAX_MS = 2000;

/**
 * PostHog standard property for session tracking.
 *
 * @see {@link https://posthog.com/docs/data/sessions}
 */
const POSTHOG_SESSION_ID_PROPERTY = "$session_id";

/**
 * PostHog property to control whether person profiles should be created.
 *
 * When false, events are tracked anonymously without creating user profiles.
 *
 * @see {@link https://posthog.com/docs/data/persons}
 */
const POSTHOG_PROCESS_PERSON_PROFILE = "$process_person_profile";

/**
 * Analytics service for tracking events using PostHog.
 *
 * This class provides a high-level interface for tracking analytics events with
 * automatic session management and anonymous tracking.
 */
export class Analytics {
  private posthog: PostHog;
  private readonly sessionId: string;
  private readonly eventPrefix: string;

  /**
   * Creates a new Analytics instance.
   *
   * @param eventPrefix - Prefix to prepend to all event names
   *
   * @example
   * ```ts
   * const analytics = new Analytics("si-tmpl");
   * ```
   */
  constructor(eventPrefix: string) {
    const apiKey = Deno.env.get("SI_POSTHOG_API_KEY") || POSTHOG_PROD_API_KEY;
    const host = Deno.env.get("SI_POSTHOG_HOST") || POSTHOG_PROD_HOST;

    this.posthog = new PostHog(apiKey, { host });
    this.sessionId = randomId();
    this.eventPrefix = eventPrefix;
  }

  /**
   * Tracks an analytics event with optional custom properties.
   *
   * This method automatically handles event naming (with prefix) and session
   * tracking. Events are tracked anonymously without creating person profiles.
   *
   * @param eventName - The event name (will be prefixed with the eventPrefix
   *   from constructor)
   * @param properties - Optional custom properties to attach to the event
   *
   * @example
   * ```ts
   * analytics.trackEvent("command-executed", { command: "generate" });
   * analytics.trackEvent("error-occurred", { errorType: "validation", code: 400 });
   * ```
   */
  trackEvent(
    eventName: string,
    properties: Record<string, unknown> = {},
  ): void {
    const event = `${this.eventPrefix}-${eventName}`;
    const distinctId = randomId();

    const payload = {
      event,
      distinctId,
      properties: {
        [POSTHOG_SESSION_ID_PROPERTY]: this.sessionId,
        [POSTHOG_PROCESS_PERSON_PROFILE]: false,
        ...properties,
      },
    };

    try {
      this.posthog.capture(payload);
    } catch (error) {
      console.error("Analytics tracking failed:", error);
    }
  }

  /**
   * Gracefully shuts down the PostHog client.
   *
   * This method ensures all pending events are flushed to PostHog before the
   * application exits. It should be called during application shutdown to
   * prevent data loss.
   *
   * @param shutdownTimeoutMs - Maximum time in milliseconds to wait for
   *   shutdown. Defaults to {@link POSTHOG_DEFAULT_SHUTDOWN_MAX_MS} (2000ms)
   *
   * @example
   * ```ts
   * // Use default timeout
   * await analytics.shutdown();
   *
   * // Custom timeout
   * await analytics.shutdown(5000);
   * ```
   */
  async shutdown(shutdownTimeoutMs?: number): Promise<void> {
    await this.posthog.shutdown(
      shutdownTimeoutMs ?? POSTHOG_DEFAULT_SHUTDOWN_MAX_MS,
    );
  }
}

/**
 * Generates a random UUID for anonymous user identification.
 *
 * Uses the Web Crypto API's randomUUID() method to generate a cryptographically
 * secure random UUID (v4).
 *
 * @returns A random UUID string
 * @internal
 */
function randomId(): string {
  return crypto.randomUUID();
}
