import { computed, reactive } from "vue";
import { defineSiStore } from "./si_store";

export const usePromptStore = defineSiStore(
  "wsNONE/admin/prompts",
  (active) => {
    // When the active state changes, reset and refetch the prompt list.
    const fetchAll = computed(() => {
      const _fetchAll = active.sdf?.get<PromptEntry[]>("v2/admin/prompts");
      // Before fetching, subscribe to updates to keep the list up to date
      active.subscribe?.("all", [
        {
          eventType: "PromptUpdated",
          callback: ({ kind, overridden }) => {
            const prompt = _fetchAll?.value?.find((p) => p.kind === kind);
            if (prompt) prompt.overridden = overridden;
          },
        },
      ]);
      return _fetchAll;
    });

    return reactive({
      get all() {
        return fetchAll.value?.value;
      },
      get(kind: PromptKind | undefined) {
        return fetchAll.value?.value?.find((p) => p.kind === kind);
      },
      get api() {
        return active.sdf?.endpoint("v2/admin/prompts");
      },
    });
  },
);

export type PromptKind = string;

export interface PromptEntry {
  kind: PromptKind;
  overridden: boolean;
}
