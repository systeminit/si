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
      :disabled="props.disabled"
      @change="updateAssociations"
    />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Assets of Type:
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="assets of type"
      :options="schemaVariantOptions"
      :disabled="props.disabled"
      @change="updateAssociations"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
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

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, props.modelValue.schemaVariantIds),
);
const selectedComponents = ref<Option[]>(
  toOptionValues(componentOptions.value, props.modelValue.componentIds),
);

const emit = defineEmits<{
  (e: "update:modelValue", v: QualificationAssocations): void;
  (e: "change", v: QualificationAssocations): void;
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
