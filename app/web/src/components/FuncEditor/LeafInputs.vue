<template>
  <div>
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Function inputs
    </h2>
    <Stack spacing="sm">
      <VormInput v-model="codeSelected" type="checkbox"> Code </VormInput>
      <VormInput v-model="deletedAtSelected" type="checkbox">
        Deleted At
      </VormInput>
      <VormInput v-model="domainSelected" type="checkbox"> Domain </VormInput>
      <VormInput v-model="resourceSelected" type="checkbox">
        Resource
      </VormInput>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { VormInput, Stack } from "@si/vue-lib/design-system";
import { LeafInputLocation } from "@/store/func/types";

const props = defineProps<{
  modelValue: LeafInputLocation[];
  disabled?: boolean;
}>();

const codeSelected = ref(props.modelValue.includes("code"));
const deletedAtSelected = ref(props.modelValue.includes("deletedAt"));
const domainSelected = ref(props.modelValue.includes("domain"));
const resourceSelected = ref(props.modelValue.includes("resource"));

watch(
  [codeSelected, deletedAtSelected, domainSelected, resourceSelected],
  ([code, deletedAt, domain, resource]) => {
    const leafInputLocations: LeafInputLocation[] = [];

    if (code) {
      leafInputLocations.push("code");
    }
    if (deletedAt) {
      leafInputLocations.push("deletedAt");
    }
    if (domain) {
      leafInputLocations.push("domain");
    }
    if (resource) {
      leafInputLocations.push("resource");
    }

    emit("update:modelValue", leafInputLocations);
    emit("change", leafInputLocations);
  },
);

const emit = defineEmits<{
  (e: "update:modelValue", v: LeafInputLocation[]): void;
  (e: "change", v: LeafInputLocation[]): void;
}>();
</script>
