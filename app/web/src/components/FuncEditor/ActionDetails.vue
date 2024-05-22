<template>
  <div class="p-3 flex flex-col gap-xs">
    <ErrorMessage :requestStatus="props.requestStatus" />
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="create"
        v-model="isCreate"
        title="This action creates a resource"
        :disabled="disabled"
        @update:model-value="setCreate"
      />
    </h2>
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="refresh"
        v-model="isRefresh"
        title="This action refreshes a resource"
        :disabled="disabled"
        @update:model-value="setRefresh"
      />
    </h2>
    <h2 class="pt-4 text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="delete"
        v-model="isDelete"
        title="This action deletes a resource"
        :disabled="disabled"
        @update:model-value="setDelete"
      />
    </h2>
    <template v-if="!schemaVariantId">
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
  </div>
</template>

<script lang="ts" setup>
import { ref, toRef, watch } from "vue";
import { storeToRefs } from "pinia";
import { ApiRequestStatus } from "@si/vue-lib/pinia";
import { ErrorMessage } from "@si/vue-lib/design-system";
import { Option } from "@/components/SelectMenu.vue";
import { ActionAssociations, FuncAssociations } from "@/store/func/types";
import SiCheckBox from "@/components/SiCheckBox.vue";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { ActionKind } from "@/store/actions.store";
import RunOnSelector from "./RunOnSelector.vue";

const funcStore = useFuncStore();
const { componentOptions, schemaVariantOptions } = storeToRefs(funcStore);

const props = defineProps<{
  modelValue: ActionAssociations;
  schemaVariantId?: string;
  disabled?: boolean;
  requestStatus: ApiRequestStatus;
}>();

const isCreate = ref(false);
const isDelete = ref(false);
const isRefresh = ref(false);
watch(
  () => props.modelValue.kind,
  () => {
    isCreate.value = props.modelValue.kind === ActionKind.Create;
    isDelete.value = props.modelValue.kind === ActionKind.Destroy;
    isRefresh.value = props.modelValue.kind === ActionKind.Refresh;
  },
  { immediate: true },
);

const setCreate = () => {
  if (!isCreate.value) return updateKind();
  isDelete.value = false;
  isRefresh.value = false;
  updateKind();
};

const setRefresh = () => {
  if (!isRefresh.value) return updateKind();
  isCreate.value = false;
  isDelete.value = false;
  updateKind();
};

const setDelete = () => {
  if (!isDelete.value) return updateKind();
  isCreate.value = false;
  isRefresh.value = false;
  updateKind();
};

const modelValue = toRef(props, "modelValue");

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, modelValue.value.schemaVariantIds),
);

const emit = defineEmits<{
  (e: "update:modelValue", v: ActionAssociations): void;
  (e: "change", v: ActionAssociations): void;
  (e: "detach", v: ActionAssociations): void;
}>();

watch(
  [modelValue, schemaVariantOptions, componentOptions],
  ([mv, svOpts]) => {
    selectedVariants.value = toOptionValues(svOpts, mv.schemaVariantIds);
  },
  { immediate: true },
);

const updateKind = () => {
  if (selectedVariants.value.length > 0) {
    updateAssociations();
  }
};

const getUpdatedAssociations = (
  schemaVariantIds: string[],
): ActionAssociations => {
  let kind = ActionKind.Manual;
  if (isCreate.value) kind = ActionKind.Create;
  if (isDelete.value) kind = ActionKind.Destroy;
  if (isRefresh.value) kind = ActionKind.Refresh;
  return {
    kind,
    schemaVariantIds,
    type: "action",
  };
};

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
