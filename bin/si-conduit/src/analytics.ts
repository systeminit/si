import { PostHog } from "posthog-node";


const POSTHOG_EVENT_PREFIX = "conduit"
const POSTHOG_SHUTDOWN_MAX_MS = 2000;

export class Analytics {
  private posthog: PostHog;
  private readonly sessionId: string;

  private readonly userId: string | undefined;
  private readonly workspaceId: string | undefined;

  constructor(authData?: { userId: string, workspaceId: string }) {
    this.sessionId = crypto.randomUUID();
    this.posthog = this.initializePostHog();

    this.userId = authData?.userId;
    this.workspaceId = authData?.workspaceId;

    if (!this.userId) return;
    this.posthog.identify({
      distinctId: this.userId,
      properties: {
        userId: this.userId,
      },
    });
  }

  private initializePostHog(): PostHog {
    const apiKey = Deno.env.get("POSTHOG_API_KEY") ||
      "phc_KpehlXOqtU44B2MeW6WjqR09NxRJCYEiUReA58QcAYK"; // Prod Posthog
    const host = Deno.env.get("POSTHOG_HOST") || "https://e.systeminit.com";

    return new PostHog(apiKey, { host });
  }

  private getDistinctId(): string | undefined {
    return this.userId;
  }
  private getWorkspaceId(): string | undefined {
    return this.workspaceId;
  }

  trackEvent(eventName: string, properties: Record<string, unknown> = {}) {
    if (!this.posthog) return;
    const event = `${POSTHOG_EVENT_PREFIX}-${eventName}`;
    try {
      const [workspaceId, userId] = [this.getWorkspaceId(), this.getDistinctId()];


      let distinctId = userId;

      const variablePayload: Record<string, unknown> = {}
      if (!distinctId || !workspaceId) {
        distinctId = crypto.randomUUID();

        variablePayload["$process_person_profile"] = false;
      } else {
        variablePayload["workspace_id"] = workspaceId;
      }

      const payload = {
        event,
        distinctId,
        properties: {
          $session_id: this.sessionId, // PostHog standard session property
          ...variablePayload,
          ...properties,
        },
      }

      console.log("Analytics tracking payload:", payload);
      this.posthog.capture(payload);
    } catch (error) {
      console.error("Analytics tracking failed:", error);
    }
  }

  async shutdown() {
    await this.posthog.shutdown(POSTHOG_SHUTDOWN_MAX_MS)
  }
}
