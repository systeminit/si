<template>
  <div class="p-3 flex flex-col gap-xs">
    <template v-if="!schemaVariantId">
      <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
        Run this code generation function on the selected components and
        component types below.
      </h1>
      <h2 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Run on Component:
      </h2>
      <RunOnSelector
        v-model="selectedComponents"
        thingLabel="components"
        :options="componentOptions"
        @change="updateAssociations"
      />
      <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Run on Schema Variant:
      </h2>
      <RunOnSelector
        v-model="selectedVariants"
        thingLabel="schema variants"
        :options="schemaVariantOptions"
        @change="updateAssociations"
      />
    </template>
    <LeafInputs v-model="inputs" @change="updateAssociations" />
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, toRef } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/components/SelectMenu.vue";
import {
  CodeGenerationAssociations,
  FuncAssociations,
} from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";
import LeafInputs from "./LeafInputs.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: CodeGenerationAssociations;
  schemaVariantId?: string;
}>();

const modelValue = toRef(props, "modelValue");

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, modelValue.value.schemaVariantIds),
);
const selectedComponents = ref<Option[]>(
  toOptionValues(componentOptions.value, modelValue.value.componentIds),
);

const inputs = ref(modelValue.value.inputs);

const emit = defineEmits<{
  (e: "update:modelValue", v: CodeGenerationAssociations): void;
  (e: "change", v: CodeGenerationAssociations): void;
}>();

watch(
  [modelValue, schemaVariantOptions, componentOptions],
  ([mv, svOpts, componentOpts]) => {
    selectedVariants.value = toOptionValues(svOpts, mv.schemaVariantIds);
    selectedComponents.value = toOptionValues(componentOpts, mv.componentIds);
    inputs.value = mv.inputs;
  },
  { immediate: true },
);

const getUpdatedAssocations = (
  schemaVariantIds: string[],
): CodeGenerationAssociations => ({
  componentIds: selectedComponents.value.map(({ value }) => value as string),
  schemaVariantIds,
  inputs: inputs.value,
  type: "codeGeneration",
});

const updateAssociations = () => {
  const associations = getUpdatedAssocations(
    selectedVariants.value.map(({ value }) => value as string),
  );
  emit("update:modelValue", associations);
  emit("change", associations);
};

const detachFunc = (): FuncAssociations | undefined => {
  if (props.schemaVariantId) {
    return getUpdatedAssocations(
      selectedVariants.value
        .map(({ value }) => value as string)
        .filter((schemaVariantId) => schemaVariantId !== props.schemaVariantId),
    );
  }
};

defineExpose({ detachFunc });
</script>
