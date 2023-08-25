<template>
  <div class="overflow-y-auto flex flex-col gap-sm w-full h-full p-sm">
    <VormInput v-model="secretFormData.name" type="text" label="Name">
      <template #instructions>
        <div class="text-neutral-700 dark:text-neutral-400 italic">
          The display name for this secret within System Initiative
        </div>
      </template>
    </VormInput>
    <VormInput
      v-model="secretFormData.description"
      type="textarea"
      label="Description"
    />
  </div>
</template>

<script setup lang="ts">
import { VormInput } from "@si/vue-lib/design-system";
import { reactive } from "vue";

const secretFormData = reactive({
  name: "",
  description: "",
});
</script>
