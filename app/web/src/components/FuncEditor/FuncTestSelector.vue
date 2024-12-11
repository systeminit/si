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
      v-model="selectedPrototypeId"
      class="w-full"
      placeholder="no binding selected"
      :options="prototypeAttributeOptions"
      :disabled="!selectedComponentId"
      checkable
      @select="setSelectedBinding"
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
const selectedPrototypeId = ref<string | undefined>(undefined);

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
    !selectedComponentId.value ||
    !props.readyToTest ||
    (props.isAttributeFunc && !selectedPrototypeId.value),
);

// Only for attribute funcs, assemble prototypes that belong to the selected component.
const prototypeAttributeOptions = computed(() => {
  let options: GroupedOptions = {};

  // Despite the fact that we can compile prototype options for _all_ components, we should wait until
  // the user selects a _single_ component.
  if (!funcStore.selectedFuncId || !selectedComponentId.value) return [];

  if (funcStore.selectedFuncSummary?.kind === FuncKind.Attribute) {
    funcStore.attributeBindings[funcStore.selectedFuncSummary.funcId]?.forEach(
      (binding) => {
        let schemaVariant;
        if (
          binding.schemaVariantId &&
          binding.schemaVariantId === props.schemaVariantId
        )
          schemaVariant =
            assetStore.variantFromListById[binding.schemaVariantId];
        if (
          binding.componentId &&
          binding.componentId === selectedComponentId.value
        ) {
          const schemaVariantId =
            componentsStore.allComponentsById[binding.componentId]?.def
              .schemaVariantId;
          if (schemaVariantId)
            schemaVariant = assetStore.variantFromListById[schemaVariantId];
        }
        if (schemaVariant) {
          options = outputSocketsAndPropsFor(schemaVariant);
        }
      },
    );

    return options;
  } else {
    // If the selected func is not an attribute func, there are no options to assemble.
    return {};
  }
});

const setSelectedComponentId = (id: string) => {
  selectedComponentId.value = id;
  loadInputIfNotAttributeFunc();
};

const setSelectedBinding = (id: string) => {
  selectedPrototypeId.value = id;
  emit("loadInput");
};

// We need the user to select a prototype after selecting a component if its an attribute func.
const loadInputIfNotAttributeFunc = () => {
  if (props.isAttributeFunc) {
    return;
  }
  emit("loadInput");
};

defineExpose({ selectedComponentId, selectedPrototypeId });

const emit = defineEmits<{
  (e: "startTest"): void;
  (e: "loadInput"): void;
}>();
</script>
