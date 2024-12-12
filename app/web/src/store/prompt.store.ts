import { computed, reactive, Ref, ref } from "vue";
import { TracingApi } from "@si/vue-lib/pinia";
import { ActiveSiStore, defineSiStore } from "./si_store";

export const usePromptStore = defineSiStore(
  `wsNONE/admin/prompts`,
  ({ sdf, subscribe }) => {
    const api = computed(() => sdf?.endpoint("v2/admin/prompts"));
    const kind = ref<PromptKind>();
    const prompts = computed(() => usePrompts(api.value, subscribe, kind));
    const text = computed(() => useText(api.value, kind.value));
    const overridden = computed(
      () =>
        prompts.value?.value?.find((p) => p.kind === kind.value)?.overridden,
    );

    return reactive({
      prompts,
      selectedPrompt: { kind, text, overridden },
      sdf,
    });
  },
);

function usePrompts(
  api: TracingApi | undefined,
  subscribe: ActiveSiStore["subscribe"] | undefined,
  kind: Ref<PromptKind | undefined>,
) {
  const prompts = api?.get<PromptEntry[]>();

  subscribe?.("all", [
    {
      eventType: "PromptUpdated",
      callback: ({ kind, overridden }) => {
        const prompt = prompts?.value?.find((p) => p.kind === kind);
        if (prompt) prompt.overridden = overridden;
      },
    },
  ]);

  // Also set the kind to the first prompt if it's not set already.
  prompts?.then((prompts) => {
    kind.value ??= prompts[0]?.kind;
  });

  return prompts;
}

// Reinitialize and refetch text whenever kind changes
function useText(api?: TracingApi, kind?: PromptKind) {
  const text = reactive({
    value: "",
    fetched: kind ? api?.get({ kind }) : undefined,
    reset() {
      return kind ? api?.delete({ kind }) : undefined;
    },
    override() {
      return kind ? api?.put({ kind, prompt_yaml: text.value }) : undefined;
    },
  });
  // Set the actual text value when it has been retrieved
  text.fetched?.then(({ prompt_yaml }) => {
    text.value = prompt_yaml;
  });
  return text;
}

type PromptKind = string;

interface PromptEntry {
  kind: PromptKind;
  overridden: boolean;
}
