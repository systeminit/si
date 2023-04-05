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
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Run this qualification on the selected asset and assets of type below.
    </h1>
    <h2 class="pt-2 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Asset:
    </h2>
    <RunOnSelector
      v-model="selectedComponents"
      thing-label="asset"
      :options="componentOptions"
      :disabled="disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Assets of Type:
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="assets of type"
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
import { QualificationAssocations } from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: QualificationAssocations;
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
  (e: "update:modelValue", v: QualificationAssocations): void;
  (e: "change", v: QualificationAssocations): void;
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
  const associations: QualificationAssocations = {
    componentIds: selectedComponents.value.map(({ value }) => value as string),
    schemaVariantIds: selectedVariants.value.map(
      ({ value }) => value as string,
    ),
    type: "qualification",
  };

  emit("update:modelValue", associations);
  emit("change", associations);
};
</script>
