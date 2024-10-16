<template>
  <div class="p-xs flex flex-col gap-xs">
    <div class="text-neutral-700 type-bold-sm dark:text-neutral-50">
      <p class="text-sm">You can detach this function above.</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, watch, computed } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/components/SelectMenu.vue";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
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

const binding = computed(() => {
  const bindings = funcStore.managementBindings[props.funcId];
  const binding = bindings
    ?.filter((b) => b.schemaVariantId === props.schemaVariantId)
    .pop();
  return binding;
});

const validSchemaVariantIds = computed(() => {
  const bindings = funcStore.managementBindings[props.funcId];
  return bindings?.map((b) => b.schemaVariantId).filter(nonNullable);
});

const selectedVariants = ref<Option[]>(
  toOptionValues(schemaVariantOptions.value, validSchemaVariantIds.value || []),
);

watch(
  [validSchemaVariantIds, schemaVariantOptions],
  () => {
    selectedVariants.value = toOptionValues(
      schemaVariantOptions.value,
      validSchemaVariantIds.value || [],
    );
  },
  { immediate: true },
);

const detachFunc = () => {
  if (binding.value) funcStore.DELETE_BINDING(props.funcId, [binding.value]);
};

defineExpose({ detachFunc });
</script>
