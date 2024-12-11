import { addStoreHooks } from "@si/vue-lib/pinia";
import { defineStore } from "pinia";
import { computed, Reactive, reactive } from "vue";
import { usePromise } from "@/utils/reactivity";
import { useRealtimeStore } from "./realtime/realtime.store";
import { useSdfApi } from "./apis";

export const usePromptStore = addStoreHooks(
  null,
  null,
  defineStore(
    `wsNONE/admin/prompts`,
    () => {
      // Create the prompts state
      const state = reactive({
        prompts: [] as PromptEntry[],
      });
      // Add in selected prompts (we pass the reactive "prompts" object in so it will be
      // fully reactive)
      return Object.assign(state, {
        selectedPrompt: useSelectedPrompt(state),
      });
    },
    {
      // On activate, refetch all the prompts, then subscribe to prompt updates
      async onActivated() {
        const api = useSdfApi({}).endpoint("v2/admin/prompts");
        this.prompts = await api.get();

        const realtimeStore = useRealtimeStore();
        return realtimeStore.subscribe(this.$id, "all", [
          {
            eventType: "PromptUpdated",
            callback: ({ kind, overridden }) => {
              const prompt = this.prompts?.find((p) => p.kind === kind);
              if (prompt) prompt.overridden = overridden;
            },
          },
        ]);
      },
    },
  ),
);

function useSelectedPrompt(promptStore: Reactive<{ prompts: PromptEntry[] }>) {
  // First get the state
  const state = reactive({
    kind: undefined as PromptKind | undefined,
    text: "",
  });

  function api() {
    if (state.kind === undefined) return undefined;
    return useSdfApi({}).endpoint("v2/admin/prompts", { kind: state.kind });
  }

  async function fetchText() {
    state.text = "";
    const fetch = api()?.get();
    if (fetch) {
      state.text = (await fetch).prompt_yaml;
      return state.text;
    }
  }

  // Add actions and computed values
  return Object.assign(state, {
    overridden: computed(
      () => promptStore.prompts.find((p) => p.kind === state.kind)?.overridden,
    ),
    override: () => api()?.put({ params: { prompt_yaml: state.text } }),
    reset: () => api()?.delete(),

    /** Fetch text whenever kind changes */
    fetchText: computed(() => usePromise(fetchText())),
    get isModified() {
      return this.fetchText.value?.isSuccess
        ? state.text !== this.fetchText.value.value
        : false;
    },
  });
}

type PromptKind = string;

interface PromptEntry {
  kind: PromptKind;
  overridden: boolean;
}
