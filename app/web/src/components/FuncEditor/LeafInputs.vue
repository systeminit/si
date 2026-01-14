<template>
  <div>
    <div class="flex flex-col gap-2xs py-xs">
      <VormInput
        v-if="kind === FuncBindingKind.Qualification"
        v-model="codeSelected"
        noLabel
        :disabled="$props.disabled"
        type="checkbox"
      >
        Code</VormInput
      >
      <VormInput v-model="deletedAtSelected" noLabel :disabled="$props.disabled" type="checkbox">
        Deleted At
      </VormInput>
      <VormInput v-model="domainSelected" noLabel :disabled="$props.disabled" type="checkbox"> Domain</VormInput>
      <VormInput v-model="resourceSelected" noLabel :disabled="$props.disabled" type="checkbox"> Resource </VormInput>
      <VormInput
        v-model="secretsSelected"
        class="pt-xs"
        label="Function Depends on Secrets"
        prompt="Run the authentication function first, to make sure the secret is applied?"
        type="radio"
        :disabled="$props.disabled"
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
import { LeafInputLocation, FuncBindingKind } from "@/api/sdf/dal/func";

const props = defineProps<{
  modelValue: LeafInputLocation[];
  disabled?: boolean;
  kind: FuncBindingKind.CodeGeneration | FuncBindingKind.Qualification;
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
  [codeSelected, deletedAtSelected, domainSelected, resourceSelected, secretsSelected],
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
