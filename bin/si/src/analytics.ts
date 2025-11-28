/**
 * Analytics Module - PostHog Integration
 *
 * This module provides analytics tracking functionality using PostHog for
 * telemetry and usage analytics. It handles event tracking with session
 * management, user identification, and workspace association.
 *
 * The module uses PostHog's standard properties and conventions for session
 * tracking and user profiling, with graceful degradation when user data is
 * unavailable (anonymous tracking).
 *
 * @example
 * ```ts
 * import { Analytics } from "./analytics.ts";
 * import type { UserData } from "./cli/jwt.ts";
 *
 * const userData: UserData = { userId: "user123", workspaceId: "ws456" };
 * const analytics = new Analytics("myapp", userData);
 *
 * // Track an event
 * analytics.trackEvent("feature-used", { feature: "export" });
 *
 * // Cleanup on exit
 * await analytics.shutdown();
 * ```
 *
 * @module
 */

import { PostHog } from "posthog-node";
import type { UserData } from "./cli/jwt.ts";

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
 * automatic session management, user identification, and workspace association.
 * It handles both authenticated and anonymous tracking scenarios.
 */
export class Analytics {
  private posthog: PostHog;
  private readonly sessionId: string;
  private readonly eventPrefix: string;

  private readonly userId: string | undefined;
  private readonly workspaceId: string | undefined;

  /**
   * Creates a new Analytics instance.
   *
   * @param eventPrefix - Prefix to prepend to all event names
   * @param userData - Optional user and workspace identification data for
   *   authenticated tracking
   *
   * @example
   * ```ts
   * // Anonymous tracking
   * const analytics = new Analytics("myapp");
   *
   * // Authenticated tracking
   * const analytics = new Analytics("myapp", { userId: "123", workspaceId: "ws456" });
   * ```
   */
  constructor(eventPrefix: string, userData?: UserData) {
    // deno-lint-ignore si-rules/no-deno-env-get -- Used only to configure analytics
    const apiKey = Deno.env.get("SI_POSTHOG_API_KEY") || POSTHOG_PROD_API_KEY;
    // deno-lint-ignore si-rules/no-deno-env-get -- Used only to configure analytics
    const host = Deno.env.get("SI_POSTHOG_HOST") || POSTHOG_PROD_HOST;

    this.posthog = new PostHog(apiKey, { host });
    this.sessionId = randomId();
    this.eventPrefix = eventPrefix;
    this.userId = userData?.userId;
    this.workspaceId = userData?.workspaceId;

    if (this.userId) {
      this.posthog.identify({
        distinctId: this.userId,
        properties: {
          userId: this.userId,
        },
      });
    }
  }

  /**
   * Tracks an analytics event with optional custom properties.
   *
   * This method automatically handles event naming (with prefix), session
   * tracking, and user identification. For anonymous users (when userData was
   * not provided), it generates a random distinct ID and disables person
   * profile creation.
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

    const [workspaceId, userId] = [this.workspaceId, this.userId];

    let distinctId = userId;

    const variablePayload: Record<string, unknown> = {};
    if (!distinctId || !workspaceId) {
      distinctId = randomId();

      variablePayload[POSTHOG_PROCESS_PERSON_PROFILE] = false;
    } else {
      variablePayload["workspace_id"] = workspaceId;
    }

    const payload = {
      event,
      distinctId,
      properties: {
        [POSTHOG_SESSION_ID_PROPERTY]: this.sessionId,
        ...variablePayload,
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
