<template>
  <Stack class="p-10 w-full">
    <h2 class="font-bold text-xl">PROMPTS</h2>
    <div class="flex flex-row gap-xs p-xs w-full">
      <select
        v-model="prompt.kind"
        class="text-neutral-900 dark:text-neutral-200 dark:bg-neutral-900 bg-neutral-100"
      >
        <option v-for="p in prompts.all" :key="p.kind" :value="p.kind">
          {{ prompt.kind }}
        </option>
      </select>
      <VButton
        v-if="prompt.kind"
        v-tooltip="'Override the prompt with the current text'"
        label="Override"
        :disabled="promptText.isDirty && prompt.isModifying"
        :requestStatus="prompt.overriding"
        loadingText="Overriding ..."
        @click="prompt.override(promptText.value)"
      />
      <!-- TODO disable unless the prompt is overridden -->
      <VButton
        v-if="prompt.kind"
        v-tooltip="'Reset the prompt to its default value'"
        label="Reset"
        :disabled="!prompt.overridden && prompt.isModifying"
        :requestStatus="prompt.resetting"
        loadingText="Resetting ..."
        @click="prompt.reset()"
      />
    </div>
    <!-- TODO show the error when fetch fails-->
    <CodeEditor
      v-if="prompt.kind"
      :id="`prompt-${prompt.kind}`"
      v-model="promptText.value"
      :recordId="prompt.kind"
      :disabled="!promptText.fetch?.isSuccess"
      yaml
    />
  </Stack>
</template>

<script lang="ts" setup>
import { Stack, VButton } from "@si/vue-lib/design-system";
import { ReactivePromise } from "@si/vue-lib/src/utils/reactivity";
import { computed, reactive, ref } from "vue";
import { PromptKind, usePromptStore } from "@/store/prompt.store";
import CodeEditor from "../CodeEditor.vue";

const prompts = usePromptStore();
const prompt = useSelectedPrompt(prompts);
const promptText = usePromptText(prompt);

function useSelectedPrompt(prompts: ReturnType<typeof usePromptStore>) {
  // Keep track of the user's selected prompt kind
  const _kind = ref<PromptKind>();

  const api = () => prompts.api?.endpoint({ kind: _kind.value });
  const overriding = ref<ReactivePromise>();
  const resetting = ref<ReactivePromise>();

  return reactive({
    get kind() {
      // Default to the first prompt
      return _kind.value ?? prompts.all?.[0]?.kind;
    },
    set kind(value: PromptKind | undefined) {
      _kind.value = value;
    },
    get overridden() {
      return prompts.get(_kind.value)?.overridden;
    },

    fetchText() {
      return api()
        ?.get<{ prompt_yaml: string }>()
        .then((p) => p.prompt_yaml);
    },
    reset() {
      if (!resetting.value) resetting.value = api()?.get();
    },
    resetting,
    override(prompt_yaml: string) {
      if (!overriding.value) overriding.value = api()?.put({ prompt_yaml });
    },
    overriding,
    get isModifying() {
      return overriding.value?.isPending || resetting.value?.isPending;
    },
  });
}

function usePromptText(prompt: ReturnType<typeof useSelectedPrompt>) {
  return computed(() => {
    const value = ref("");
    return reactive({
      value,
      // eslint-disable-next-line vue/no-async-in-computed-properties
      fetch: prompt.fetchText()?.then((text) => {
        value.value = text;
      }),
      get isDirty() {
        return value.value !== prompt.fetchText()?.value;
      },
    });
  });
}
</script>
