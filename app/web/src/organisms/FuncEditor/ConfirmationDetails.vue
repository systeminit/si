<template>
  <div class="p-3 flex flex-col gap-2">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Run this confirmation on the selected components and component types
      below.
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
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/molecules/SelectMenu.vue";
import { ConfirmationAssociations } from "@/store/func/types";
import { toOptionValues } from "@/organisms/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: ConfirmationAssociations;
  disabled?: boolean;
}>();

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, props.modelValue.schemaVariantIds),
);
const selectedComponents = ref<Option[]>(
  toOptionValues(componentOptions.value, props.modelValue.componentIds),
);

const emit = defineEmits<{
  (e: "update:modelValue", v: ConfirmationAssociations): void;
  (e: "change", v: ConfirmationAssociations): void;
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
  },
  { immediate: true },
);

const updateAssociations = () => {
  const associations: ConfirmationAssociations = {
    componentIds: selectedComponents.value.map(({ value }) => value as number),
    schemaVariantIds: selectedVariants.value.map(
      ({ value }) => value as number,
    ),
    type: "confirmation",
  };

  emit("update:modelValue", associations);
  emit("change", associations);
};
</script>
