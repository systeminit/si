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
      :options="components"
      :disabled="props.disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Schema Variant:
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="schema variants"
      :options="schemaVariants"
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
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { CodeGenerationAssociations } from "@/service/func";
import { toOptionValues } from "@/organisms/FuncEditor/utils";
import { CodeLanguage } from "@/api/sdf/dal/code_view";
import RunOnSelector from "./RunOnSelector.vue";

const props = defineProps<{
  schemaVariants: Option[];
  components: Option[];
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

const selectedVariants = ref<Option[]>(
  toOptionValues(props.schemaVariants, props.modelValue.schemaVariantIds),
);
const selectedComponents = ref<Option[]>(
  toOptionValues(props.components, props.modelValue.componentIds),
);
const selectedFormat = ref<Option>(
  formatOptions.find(({ value }) => value === props.modelValue.format) ??
    formatOptions[2],
);

const emit = defineEmits<{
  (e: "update:modelValue", v: CodeGenerationAssociations): void;
  (e: "change", v: CodeGenerationAssociations): void;
}>();

watch(
  () => props.modelValue,
  (mv) => {
    selectedVariants.value = toOptionValues(
      props.schemaVariants,
      mv.schemaVariantIds,
    );
    selectedComponents.value = toOptionValues(
      props.components,
      mv.componentIds,
    );
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
