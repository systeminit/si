<template>
  <div class="flex flex-col items-center gap-xs">
    <DropdownMenuButton
      v-model="selectedComponentId"
      class="w-full"
      placeholder="no component selected"
      :options="componentAttributeOptions"
      checkable
      @select="setSelectedComponentId"
    />
    <DropdownMenuButton
      v-if="isAttributeFunc"
      v-model="selectedOutputLocationId"
      :options="outputLocationAttributeOptions"
      class="w-full"
      placeholder="no output location selected"
      checkable
      :disabled="!selectedComponentId"
      @select="setSelectedOutputLocation"
    />
    <VButton
      class="w-full"
      label="Run Test"
      size="sm"
      :loading="testStatus === 'running'"
      loadingText="Running"
      loadingIcon="loader"
      icon="play"
      :disabled="disableTestButton"
      @click="emit('startTest')"
    />
  </div>
</template>

<script lang="ts" setup>
import { DropdownMenuButton, VButton } from "@si/vue-lib/design-system";
import { PropType, computed, ref } from "vue";
import { useComponentsStore } from "@/store/components.store";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { FuncKind } from "@/api/sdf/dal/func";
import { outputSocketsAndPropsFor } from "@/api/sdf/dal/schema";
import { GroupedOptions } from "@/components/SelectMenu.vue";
import { TestStatus } from "./FuncTest.vue";

const componentsStore = useComponentsStore();
const assetStore = useAssetStore();
const funcStore = useFuncStore();

const props = defineProps({
  testStatus: { type: String as PropType<TestStatus>, required: true },
  schemaVariantId: { type: String, required: true },
  readyToTest: { type: Boolean, required: true },
  isAttributeFunc: { type: Boolean, required: true },
});

const selectedComponentId = ref<string | undefined>(undefined);
const selectedOutputLocationId = ref<string | undefined>(undefined);

const componentAttributeOptions = computed(() => {
  return componentsForSchemaVariantId.value.map((c) => {
    return { value: c.def.id, label: c.def.displayName };
  });
});
const componentsForSchemaVariantId = computed(() => {
  return Object.values(componentsStore.allComponentsById).filter(
    (c) => c.def.schemaVariantId === props.schemaVariantId,
  );
});

const disableTestButton = computed(
  (): boolean =>
    !selectedComponentId.value || !props.readyToTest || (props.isAttributeFunc && !selectedOutputLocationId.value),
);

const outputLocationAttributeOptions = computed(() => {
  let options: GroupedOptions = {};

  if (!funcStore.selectedFuncId || !selectedComponentId.value) return options;
  if (funcStore.selectedFuncSummary?.kind !== FuncKind.Attribute) return options;

  funcStore.attributeBindings[funcStore.selectedFuncSummary.funcId]?.forEach((binding) => {
    let schemaVariant;
    if (binding.schemaVariantId && binding.schemaVariantId === props.schemaVariantId)
      schemaVariant = assetStore.variantFromListById[binding.schemaVariantId];
    if (binding.componentId && binding.componentId === selectedComponentId.value) {
      const schemaVariantId = componentsStore.allComponentsById[binding.componentId]?.def.schemaVariantId;
      if (schemaVariantId) schemaVariant = assetStore.variantFromListById[schemaVariantId];
    }
    if (schemaVariant) {
      options = outputSocketsAndPropsFor(schemaVariant);
    }
  });

  // NOTE(nick): this is a bit cursed, but we need to flatten the results to work with the current
  // state of the func test panel. We will "fix" the label accordingly.
  const processedOptions = [];
  for (const [groupLabel, innerOptions] of Object.entries(options)) {
    for (const innerOption of innerOptions) {
      processedOptions.push({
        label: groupLabel === "Output Sockets" ? innerOption.label : `/${groupLabel}${innerOption.label}`,
        value: innerOption.value,
      });
    }
  }
  return processedOptions;
});

const setSelectedComponentId = (id: string) => {
  selectedComponentId.value = id;
  loadInputIfNotAttributeFunc();
};

const setSelectedOutputLocation = (id: string) => {
  selectedOutputLocationId.value = id;
  emit("loadInput");
};

// We need the user to select a prototype after selecting a component if its an attribute func.
const loadInputIfNotAttributeFunc = () => {
  if (props.isAttributeFunc) {
    return;
  }
  emit("loadInput");
};

defineExpose({ selectedComponentId, selectedOutputLocationId });

const emit = defineEmits<{
  (e: "startTest"): void;
  (e: "loadInput"): void;
}>();
</script>
