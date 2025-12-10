import { PostHog } from "posthog-node";
import { Context } from "../../context.ts";

export class Analytics {
  private posthog: PostHog | null = null;
  private sessionId: string;

  constructor() {
    this.sessionId = crypto.randomUUID();
    this.initializePostHog();
  }

  private initializePostHog() {
    const apiKey =
      // deno-lint-ignore si-rules/no-deno-env-get
      Deno.env.get("POSTHOG_API_KEY") ||
      "phc_KpehlXOqtU44B2MeW6WjqR09NxRJCYEiUReA58QcAYK"; // Prod Posthog
    // deno-lint-ignore si-rules/no-deno-env-get
    const host = Deno.env.get("POSTHOG_HOST") || "https://e.systeminit.com";

    if (apiKey) {
      this.posthog = new PostHog(apiKey, { host });
    }
  }

  private getDistinctId(): string {
    return Context.userId();
  }
  private getWorkspaceId(): string {
    return Context.workspaceId();
  }

  identifyUser() {
    if (!this.posthog) return;
    this.posthog.identify({
      distinctId: this.getDistinctId(),
      properties: {
        userId: this.getDistinctId(),
      },
    });
  }

  trackEvent(eventName: string, properties: Record<string, unknown> = {}) {
    if (!this.posthog) return;
    const event = `mcp-${eventName}`;
    try {
      this.posthog.capture({
        distinctId: this.getDistinctId(),
        event,
        properties: {
          $session_id: this.sessionId, // PostHog standard session property
          workspace_id: this.getWorkspaceId(),
          ...properties,
        },
      });
    } catch (error) {
      console.error("Analytics tracking failed:", error);
    }
  }

  trackToolUsage(toolName: string, executionTimeMs: number) {
    this.trackEvent("tool_used", {
      toolName,
      executionTimeMs,
    });
  }

  trackError(toolName: string, errorProperties?: Record<string, unknown>) {
    this.trackEvent("tool_error", {
      toolName,
      ...errorProperties,
    });
  }

  trackServerStart() {
    this.trackEvent("server_started");
  }

  trackServerEnd() {
    this.trackEvent("server_ended");
    if (this.posthog) {
      this.posthog.shutdown();
    }
  }
}

export const analytics = new Analytics();
