<template>
  <div class="p-3 flex flex-col gap-2">
    <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
      Run this confirmation on the selected components and component types
      below.
    </h1>
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      Run on Assets of Type
    </h2>
    <RunOnSelector
      v-model="selectedVariants"
      thing-label="asset type"
      :options="schemaVariantOptions"
      :disabled="props.disabled"
      @change="updateAssociations"
    />
    <VButton @click="openModal">Edit Confirmation Descriptions</VButton>
    <ConfirmationDescriptionModal
      ref="descriptionsModal"
      v-model="funcDescriptions"
      :schema-variants="selectedVariants"
      @change="updateAssociations"
    />
  </div>
</template>

<script lang="ts" setup>
import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { Modal, VButton } from "@si/vue-lib/design-system";
import { Option } from "@/components/SelectMenu.vue";
import {
  ConfirmationAssociations,
  FuncDescriptionView,
} from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";
import ConfirmationDescriptionModal from "./ConfirmationDescriptionModal.vue";

const funcStore = useFuncStore();
const { schemaVariantOptions } = storeToRefs(funcStore);

const descriptionsModal = ref<InstanceType<typeof Modal>>();

const props = defineProps<{
  modelValue: ConfirmationAssociations;
  disabled?: boolean;
}>();

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, props.modelValue.schemaVariantIds),
);

const funcDescriptions = ref<FuncDescriptionView[]>(
  props.modelValue.descriptions,
);

const emit = defineEmits<{
  (e: "update:modelValue", v: ConfirmationAssociations): void;
  (e: "change", v: ConfirmationAssociations): void;
}>();

const openModal = () => {
  descriptionsModal?.value?.open();
};

watch(
  () => props.modelValue,
  (mv) => {
    selectedVariants.value = toOptionValues(
      schemaVariantOptions.value,
      mv.schemaVariantIds,
    );
  },
  { immediate: true },
);

const updateAssociations = () => {
  const associations: ConfirmationAssociations = {
    componentIds: props.modelValue.componentIds,
    schemaVariantIds: selectedVariants.value.map(
      ({ value }) => value as string,
    ),
    descriptions: funcDescriptions.value,
    type: "confirmation",
  };

  emit("update:modelValue", associations);
  emit("change", associations);
};
</script>
