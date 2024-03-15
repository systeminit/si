<template>
  <Stack class="h-full w-full my-4 mx-4">
    <VormInput
      v-model="selectedSchemaVariant"
      label="Schema Variants"
      type="dropdown"
      class="flex-1"
      :options="schemaVariantOptions"
    />

    <WorkspaceVizSchemaVariant
      :key="selectedSchemaVariant"
      :schemaVariantId="selectedSchemaVariant"
    />
  </Stack>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import { VormInput, Stack } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import WorkspaceVizSchemaVariant from "./WorkspaceVizSchemaVariant.vue";

const componentStore = useComponentsStore();

const selectedSchemaVariant = ref();
const schemaVariantOptions = computed(() =>
  componentStore.schemaVariants.map((sv) => ({
    label: sv.schemaName + (sv.builtin ? " (builtin)" : ""),
    value: sv.id,
  })),
);
</script>
