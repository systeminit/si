<template>
  <div class="p-xs flex flex-col gap-xs">
    <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="create"
        v-model="isCreate"
        title="This action creates a resource"
        :disabled="disabled || func?.isLocked"
        @update:model-value="setCreate"
      />
    </div>
    <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="refresh"
        v-model="isRefresh"
        title="This action refreshes a resource"
        :disabled="disabled || func?.isLocked"
        @update:model-value="setRefresh"
      />
    </div>
    <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="delete"
        v-model="isDelete"
        title="This action deletes a resource"
        :disabled="disabled || func?.isLocked"
        @update:model-value="setDelete"
      />
    </div>
    <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      <SiCheckBox
        id="update"
        v-model="isUpdate"
        title="This action updates a resource"
        :disabled="disabled || func?.isLocked"
        @update:model-value="setUpdate"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, computed } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/components/SelectMenu.vue";
import SiCheckBox from "@/components/SiCheckBox.vue";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { ActionKind } from "@/api/sdf/dal/action";
import { FuncId } from "@/api/sdf/dal/func";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { nonNullable } from "@/utils/typescriptLinter";

const funcStore = useFuncStore();
const assetStore = useAssetStore();
const { schemaVariantOptions } = storeToRefs(assetStore);

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId: SchemaVariantId;
  disabled?: boolean;
}>();

const isCreate = ref(false);
const isDelete = ref(false);
const isRefresh = ref(false);
const isUpdate = ref(false);

const func = computed(() => {
  return funcStore.funcsById[props.funcId];
});

const binding = computed(() => {
  const bindings = funcStore.actionBindings[props.funcId];
  const binding = bindings?.filter((b) => b.schemaVariantId === props.schemaVariantId).pop();
  return binding;
});

const validSchemaVariantIds = computed(() => {
  const bindings = funcStore.actionBindings[props.funcId];
  return bindings?.map((b) => b.schemaVariantId).filter(nonNullable);
});

watch(
  binding,
  () => {
    isCreate.value = binding.value?.kind === ActionKind.Create;
    isDelete.value = binding.value?.kind === ActionKind.Destroy;
    isRefresh.value = binding.value?.kind === ActionKind.Refresh;
    isUpdate.value = binding.value?.kind === ActionKind.Update;
  },
  { immediate: true },
);

const setCreate = () => {
  if (!isCreate.value) return updateKind();
  isDelete.value = false;
  isRefresh.value = false;
  isUpdate.value = false;
  updateKind();
};

const setRefresh = () => {
  if (!isRefresh.value) return updateKind();
  isCreate.value = false;
  isDelete.value = false;
  isUpdate.value = false;
  updateKind();
};

const setDelete = () => {
  if (!isDelete.value) return updateKind();
  isCreate.value = false;
  isRefresh.value = false;
  isUpdate.value = false;
  updateKind();
};

const setUpdate = () => {
  if (!isUpdate.value) return updateKind();
  isCreate.value = false;
  isDelete.value = false;
  isRefresh.value = false;
  updateKind();
};

const selectedVariants = ref<Option[]>(toOptionValues(schemaVariantOptions.value, validSchemaVariantIds.value || []));

watch(
  [validSchemaVariantIds, schemaVariantOptions],
  () => {
    selectedVariants.value = toOptionValues(schemaVariantOptions.value, validSchemaVariantIds.value || []);
  },
  { immediate: true },
);

const updateKind = () => {
  if (binding.value) {
    binding.value.kind = ActionKind.Manual;
    if (isCreate.value) binding.value.kind = ActionKind.Create;
    if (isDelete.value) binding.value.kind = ActionKind.Destroy;
    if (isRefresh.value) binding.value.kind = ActionKind.Refresh;
    if (isUpdate.value) binding.value.kind = ActionKind.Update;
    funcStore.UPDATE_BINDING(props.funcId, [binding.value]);
  }
};

const detachFunc = () => {
  if (binding.value) funcStore.DELETE_BINDING(props.funcId, [binding.value]);
};

defineExpose({ detachFunc });
</script>
