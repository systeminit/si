<template>
  <div>
    <h2 class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      Function inputs
    </h2>
    <div class="flex flex-col gap-2xs py-xs">
      <VormInput v-model="codeSelected" noLabel type="checkbox">
        Code</VormInput
      >
      <VormInput v-model="deletedAtSelected" noLabel type="checkbox">
        Deleted At
      </VormInput>
      <VormInput v-model="domainSelected" noLabel type="checkbox">
        Domain</VormInput
      >
      <VormInput v-model="resourceSelected" noLabel type="checkbox">
        Resource
      </VormInput>
      <VormInput
        v-model="secretsSelected"
        class="pt-xs"
        label="Function Depends on Secrets"
        prompt="Run the authentication function first, to make sure the secret is applied?"
        type="radio"
        :options="[
          { value: true, label: 'Yes' },
          { value: false, label: 'No' },
        ]"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { VormInput } from "@si/vue-lib/design-system";
import { LeafInputLocation } from "@/api/sdf/dal/func";

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
