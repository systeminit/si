<template>
  <div class="p-3 flex flex-col gap-2">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Run this code generation function on the selected components and component
      types below.
    </h1>
    <h2 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Component:
    </h2>
    <RunOnSelector
      v-model="selectedComponents"
      thing-label="components"
      :options="componentOptions"
      :disabled="disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Schema Variant:
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="schema variants"
      :options="schemaVariantOptions"
      :disabled="disabled"
      @change="updateAssociations"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, toRef } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/components/SelectMenu.vue";
import { CodeGenerationAssociations } from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: CodeGenerationAssociations;
  disabled?: boolean;
}>();

const modelValue = toRef(props, "modelValue");

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, modelValue.value.schemaVariantIds),
);
const selectedComponents = ref<Option[]>(
  toOptionValues(componentOptions.value, modelValue.value.componentIds),
);

const emit = defineEmits<{
  (e: "update:modelValue", v: CodeGenerationAssociations): void;
  (e: "change", v: CodeGenerationAssociations): void;
}>();

watch(
  [modelValue, schemaVariantOptions, componentOptions],
  ([mv, svOpts, componentOpts]) => {
    selectedVariants.value = toOptionValues(svOpts, mv.schemaVariantIds);
    selectedComponents.value = toOptionValues(componentOpts, mv.componentIds);
  },
  { immediate: true },
);

const updateAssociations = () => {
  const associations: CodeGenerationAssociations = {
    componentIds: selectedComponents.value.map(({ value }) => value as string),
    schemaVariantIds: selectedVariants.value.map(
      ({ value }) => value as string,
    ),
    type: "codeGeneration",
  };

  emit("update:modelValue", associations);
  emit("change", associations);
};
</script>
