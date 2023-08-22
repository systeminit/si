<template>
  <div class="p-3 flex flex-col gap-2">
    <p class="text-neutral-400 dark:text-neutral-300 text-sm">
      For more information on authoring a qualification, please read the
      <a
        href="http://systeminit.com/docs/qualifications"
        target="_blank"
        class="hover:underline"
        >qualification documentation
      </a>
    </p>
    <template v-if="!schemaVariantId">
      <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
        Run this qualification on the selected asset and assets of type below.
      </h1>
      <h2 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Run on Asset:
      </h2>
      <RunOnSelector
        v-model="selectedComponents"
        thingLabel="asset"
        :options="componentOptions"
        :disabled="disabled"
        @change="updateAssociations"
      />
      <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Run on Assets of Type:
      </h2>
      <RunOnSelector
        v-model="selectedVariants"
        thingLabel="assets of type"
        :options="schemaVariantOptions"
        :disabled="disabled"
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
  FuncAssociations,
  QualificationAssociations,
} from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";
import LeafInputs from "./LeafInputs.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: QualificationAssociations;
  schemaVariantId?: string;
  disabled?: boolean;
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
  (e: "update:modelValue", v: QualificationAssociations): void;
  (e: "change", v: QualificationAssociations): void;
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
): QualificationAssociations => ({
  componentIds: selectedComponents.value.map(({ value }) => value as string),
  schemaVariantIds,
  inputs: inputs.value,
  type: "qualification",
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
