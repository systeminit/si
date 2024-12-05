<template>
  <Stack class="p-10 w-full">
    <h2 class="font-bold text-xl">PROMPTS</h2>
    <div class="flex flex-row gap-xs p-xs w-full">
      <select
        v-model="promptKind"
        class="text-neutral-900 dark:text-neutral-200 dark:bg-neutral-900 bg-neutral-100"
      >
        <option
          v-for="prompt in prompts.prompts"
          :key="prompt.kind"
          :value="prompt.kind"
        >
          {{ prompt.kind }}
        </option>
      </select>
      <VButton
        v-if="promptKind"
        v-tooltip="'Override the prompt with the current text'"
        label="Override"
        :disabled="isBusy"
        :requestStatus="overrideStatus"
        loadingText="Overriding ..."
        @click="prompts.OVERRIDE_PROMPT(promptKind, promptText)"
      />
      <!-- TODO disable unless the prompt is overridden -->
      <VButton
        v-if="promptKind"
        v-tooltip="'Reset the prompt to its default value'"
        label="Reset"
        :disabled="isBusy || !promptOverridden"
        :requestStatus="resetStatus"
        loadingText="Resetting ..."
        @click="prompts.RESET_PROMPT(promptKind)"
      />
    </div>
    <CodeEditor
      v-if="promptKind"
      :id="`prompt-${promptKind}`"
      v-model="promptText"
      :recordId="promptKind"
      :disabled="!fetchStatus.isSuccess"
      yaml
    />
  </Stack>
</template>

<script lang="ts" setup>
import { storeToRefs } from "pinia";
import { computed } from "vue";
import { Stack, VButton } from "@si/vue-lib/design-system";
import { usePromptStore } from "@/store/prompt.store";
import CodeEditor from "../CodeEditor.vue";

const prompts = usePromptStore();
const {
  selectedPromptKind: promptKind,
  selectedPromptText: promptText,
  selectedPromptOverridden: promptOverridden,
} = storeToRefs(prompts);
const fetchStatus = prompts.getRequestStatus("FETCH_PROMPT", promptKind);
const overrideStatus = prompts.getRequestStatus("OVERRIDE_PROMPT", promptKind);
const resetStatus = prompts.getRequestStatus("RESET_PROMPT", promptKind);
const isBusy = computed(
  () =>
    fetchStatus.value.isPending ||
    overrideStatus.value.isPending ||
    resetStatus.value.isPending,
);
</script>
