<template>
  <div>
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Function inputs
    </h2>
    <Stack spacing="sm">
      <VormInput v-model="codeSelected" type="checkbox"> Code</VormInput>
      <VormInput v-model="deletedAtSelected" type="checkbox">
        Deleted At
      </VormInput>
      <VormInput v-model="domainSelected" type="checkbox"> Domain</VormInput>
      <VormInput v-model="resourceSelected" type="checkbox">
        Resource
      </VormInput>
      <VormInput
        v-model="secretsSelected"
        label="Function Depends on Secrets"
        prompt="Run the authentication function first, to make sure the secret is applied?"
        type="radio"
        :options="[
          { value: true, label: 'Yes' },
          { value: false, label: 'No' },
        ]"
      />
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { Stack, VormInput } from "@si/vue-lib/design-system";
import { LeafInputLocation } from "@/store/func/types";

const props = defineProps<{
  modelValue: LeafInputLocation[];
  disabled?: boolean;
}>();

const codeSelected = ref(props.modelValue.includes("code"));
const deletedAtSelected = ref(props.modelValue.includes("deletedAt"));
const domainSelected = ref(props.modelValue.includes("domain"));
const resourceSelected = ref(props.modelValue.includes("resource"));
const secretsSelected = ref(props.modelValue.includes("secrets"));

watch(
  () => props.modelValue,
  (inputs) => {
    codeSelected.value = inputs.includes("code");
    deletedAtSelected.value = inputs.includes("deletedAt");
    domainSelected.value = inputs.includes("domain");
    resourceSelected.value = inputs.includes("resource");
    secretsSelected.value = inputs.includes("secrets");
  },
);

watch(
  [
    codeSelected,
    deletedAtSelected,
    domainSelected,
    resourceSelected,
    secretsSelected,
  ],
  ([code, deletedAt, domain, resource, secrets]) => {
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
    if (secrets) {
      leafInputLocations.push("secrets");
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
