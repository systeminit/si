import { useChangeSetsStore } from "@/store/change_sets.store";
import { posthog } from "./posthog";

export function trackEvent(
  eventName: string,
  properties?: Record<string, any>,
) {
  const changeSetStore = useChangeSetsStore();
  posthog.capture(`wa-${eventName}`, {
    workspace_id: changeSetStore.selectedWorkspacePk,
    changeset_id: changeSetStore.selectedChangeSetId,
    ...properties,
  });
}
