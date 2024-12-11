<template>
  <Stack class="p-10 w-full">
    <h2 class="font-bold text-xl">PROMPTS</h2>
    <div class="flex flex-row gap-xs p-xs w-full">
      <select
        v-model="prompt.kind"
        class="text-neutral-900 dark:text-neutral-200 dark:bg-neutral-900 bg-neutral-100"
      >
        <option v-for="p in prompts.prompts" :key="p.kind" :value="p.kind">
          {{ prompt.kind }}
        </option>
      </select>
      <VButton
        v-if="prompt.kind"
        v-tooltip="'Override the prompt with the current text'"
        label="Override"
        :disabled="overrideStatus?.isPending || resetStatus?.isPending"
        :requestStatus="overrideStatus"
        loadingText="Overriding ..."
        @click="overrideStatus = usePromise(prompt.override())"
      />
      <!-- TODO disable unless the prompt is overridden -->
      <VButton
        v-if="prompt.kind"
        v-tooltip="'Reset the prompt to its default value'"
        label="Reset"
        :disabled="
          overrideStatus?.isPending ||
          resetStatus?.isPending ||
          !prompt.overridden
        "
        :requestStatus="resetStatus"
        loadingText="Resetting ..."
        @click="resetStatus = usePromise(prompt.reset())"
      />
    </div>
    <!-- TODO show the error when fetch fails-->
    <CodeEditor
      v-if="prompt.kind"
      :id="`prompt-${prompt.kind}`"
      v-model="prompt.text"
      :recordId="prompt.kind"
      :disabled="!prompt.fetchText?.isSuccess"
      yaml
    />
  </Stack>
</template>

<script lang="ts" setup>
import { Stack, VButton } from "@si/vue-lib/design-system";
import { ref } from "vue";
import { usePromptStore } from "@/store/prompt.store";
import { UsePromise, usePromise } from "@/utils/reactivity";
import CodeEditor from "../CodeEditor.vue";

const prompts = usePromptStore();
const prompt = prompts.selectedPrompt;
const overrideStatus = ref<UsePromise>();
const resetStatus = ref<UsePromise>();
</script>
