<template>
  <div class="p-3 flex flex-col gap-2">
    <template v-if="!schemaVariantId">
      <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
        Run this code generation function on the selected components and
        component types below.
      </h1>
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
  </div>
</template>

<script lang="ts" setup>
import { ref, toRef, watch } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/components/SelectMenu.vue";
import {
  AuthenticationAssociations,
  FuncAssociations,
} from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: AuthenticationAssociations;
  schemaVariantId?: string;
}>();

const modelValue = toRef(props, "modelValue");

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, modelValue.value.schemaVariantIds),
);

const emit = defineEmits<{
  (e: "update:modelValue", v: AuthenticationAssociations): void;
  (e: "change", v: AuthenticationAssociations): void;
}>();

watch(
  [modelValue, schemaVariantOptions, componentOptions],
  ([mv, svOpts]) => {
    selectedVariants.value = toOptionValues(svOpts, mv.schemaVariantIds);
  },
  { immediate: true },
);

const getUpdatedAssociations = (
  schemaVariantIds: string[],
): AuthenticationAssociations => ({
  schemaVariantIds,
  type: "authentication",
});

const updateAssociations = () => {
  const associations = getUpdatedAssociations(
    selectedVariants.value.map(({ value }) => value as string),
  );
  emit("update:modelValue", associations);
  emit("change", associations);
};

const detachFunc = (): FuncAssociations | undefined => {
  if (props.schemaVariantId) {
    return getUpdatedAssociations(
      selectedVariants.value
        .map(({ value }) => value as string)
        .filter((schemaVariantId) => schemaVariantId !== props.schemaVariantId),
    );
  }
};

defineExpose({ detachFunc });
</script>
