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
      :disabled="props.disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Schema Variant:
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="schema variants"
      :options="schemaVariantOptions"
      :disabled="props.disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Type of code generated:
    </h2>
    <SelectMenu
      v-model="selectedFormat"
      class="w-4/5"
      :options="formatOptions"
      :disabledi="props.disabled"
      @change="updateAssociations"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { CodeGenerationAssociations } from "@/store/func/types";
import { toOptionValues } from "@/organisms/FuncEditor/utils";
import { CodeLanguage } from "@/api/sdf/dal/code_view";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: CodeGenerationAssociations;
  disabled?: boolean;
}>();

const formatOptions: Option[] = [
  {
    label: "Diff",
    value: "diff",
  },
  {
    label: "Json",
    value: "json",
  },
  {
    label: "Unknown",
    value: "unknown",
  },
  {
    label: "YAML",
    value: "yaml",
  },
];

const getFormatOption = (format: CodeLanguage): Option =>
  formatOptions.find(({ value }) => value === (format as string)) ??
  formatOptions[2];

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, props.modelValue.schemaVariantIds),
);
const selectedComponents = ref<Option[]>(
  toOptionValues(componentOptions.value, props.modelValue.componentIds),
);
const selectedFormat = ref<Option>(getFormatOption(props.modelValue.format));

const emit = defineEmits<{
  (e: "update:modelValue", v: CodeGenerationAssociations): void;
  (e: "change", v: CodeGenerationAssociations): void;
}>();

watch(
  () => props.modelValue,
  (mv) => {
    selectedVariants.value = toOptionValues(
      schemaVariantOptions.value,
      mv.schemaVariantIds,
    );
    selectedComponents.value = toOptionValues(
      componentOptions.value,
      mv.componentIds,
    );
    selectedFormat.value = getFormatOption(props.modelValue.format);
  },
  { immediate: true },
);

const updateAssociations = () => {
  const associations: CodeGenerationAssociations = {
    componentIds: selectedComponents.value.map(({ value }) => value as number),
    schemaVariantIds: selectedVariants.value.map(
      ({ value }) => value as number,
    ),
    format: selectedFormat.value.value as CodeLanguage,
    type: "codeGeneration",
  };

  emit("update:modelValue", associations);
  emit("change", associations);
};
</script>
