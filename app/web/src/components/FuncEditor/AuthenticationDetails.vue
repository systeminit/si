<template>
  <div class="p-3 flex flex-col gap-xs">
    <p class="text-sm">You can detach this function above.</p>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed, watch } from "vue";
import { storeToRefs } from "pinia";
import { Option } from "@/components/SelectMenu.vue";
import { FuncId } from "@/api/sdf/dal/func";
import { SchemaVariantId } from "@/api/sdf/dal/schema";
import { toOptionValues } from "@/components/FuncEditor/utils";
import { useFuncStore } from "@/store/func/funcs.store";
import { useAssetStore } from "@/store/asset.store";
import { nonNullable } from "@/utils/typescriptLinter";

const funcStore = useFuncStore();
const assetStore = useAssetStore();
const { schemaVariantOptions } = storeToRefs(assetStore);

const props = defineProps<{
  funcId: FuncId;
  schemaVariantId?: SchemaVariantId;
}>();

const binding = computed(() => {
  const bindings = funcStore.authenticationBindings[props.funcId];
  const binding = bindings?.filter((b) => b.schemaVariantId === props.schemaVariantId).pop();
  return binding;
});

const validSchemaVariantIds = computed(() => {
  const bindings = funcStore.actionBindings[props.funcId];
  return bindings?.map((b) => b.schemaVariantId).filter(nonNullable);
});

const selectedVariants = ref<Option[]>(toOptionValues(schemaVariantOptions.value, validSchemaVariantIds.value || []));

watch(
  [validSchemaVariantIds, schemaVariantOptions],
  () => {
    selectedVariants.value = toOptionValues(schemaVariantOptions.value, validSchemaVariantIds.value || []);
  },
  { immediate: true },
);

const detachFunc = () => {
  if (binding.value) funcStore.DELETE_BINDING(props.funcId, [binding.value]);
};

defineExpose({ detachFunc });
</script>
