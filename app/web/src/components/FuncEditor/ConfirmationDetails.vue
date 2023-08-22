<template>
  <div class="p-3 flex flex-col gap-2">
    <template v-if="!schemaVariantId">
      <h1 class="text-neutral-400 dark:text-neutral-300 text-sm">
        Run this confirmation on the selected components and component types
        below.
      </h1>
      <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
        Run on Assets of Type
      </h2>
      <RunOnSelector
        v-model="selectedVariants"
        thingLabel="asset type"
        :options="schemaVariantOptions"
        :disabled="props.disabled"
        @change="updateAssociations"
      />
    </template>
    <LeafInputs v-model="inputs" @change="updateAssociations" />
    <VButton @click="openModal">Edit Confirmation Descriptions</VButton>
    <ConfirmationDescriptionModal
      ref="descriptionsModal"
      v-model="funcDescriptions"
      :schemaVariants="selectedVariants"
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
  FuncAssociations,
  FuncDescriptionView,
} from "@/store/func/types";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import RunOnSelector from "./RunOnSelector.vue";
import ConfirmationDescriptionModal from "./ConfirmationDescriptionModal.vue";
import LeafInputs from "./LeafInputs.vue";

const props = defineProps<{
  modelValue: ConfirmationAssociations;
  schemaVariantId?: string;
  disabled?: boolean;
}>();

const funcStore = useFuncStore();
const { schemaVariantOptions } = storeToRefs(funcStore);

const descriptionsModal = ref<InstanceType<typeof Modal>>();

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, props.modelValue.schemaVariantIds),
);

const funcDescriptions = ref<FuncDescriptionView[]>(
  props.modelValue.descriptions,
);

const inputs = ref(props.modelValue.inputs);

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
    inputs.value = mv.inputs;
  },
  { immediate: true },
);

const getUpdatedAssocations = (
  schemaVariantIds: string[],
): ConfirmationAssociations => ({
  componentIds: props.modelValue.componentIds,
  schemaVariantIds,
  inputs: inputs.value,
  descriptions: funcDescriptions.value,
  type: "confirmation",
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
