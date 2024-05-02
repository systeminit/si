<template>
  <div class="flex flex-row items-center gap-sm">
    <VormInput
      v-model="selectedComponentId"
      class="flex-grow"
      type="dropdown"
      placeholder="no component selected"
      noLabel
      :options="componentAttributeOptions"
      @update:model-value="loadInputIfNotAttributeFunc"
    />
    <VormInput
      v-if="isAttributeFunc"
      v-model="selectedPrototypeId"
      class="flex-grow"
      type="dropdown"
      placeholder="no binding selected"
      noLabel
      :disabled="!selectedComponentId"
      :options="prototypeAttributeOptions"
      @update:model-value="emit('loadInput')"
    />
    <VButton
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
import { VButton, VormInput } from "@si/vue-lib/design-system";
import { PropType, computed, ref } from "vue";
import { useComponentsStore } from "@/store/components.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { AttributePrototypeBag } from "@/store/func/types";
import { TestStatus } from "./FuncTest.vue";

const componentsStore = useComponentsStore();
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
    return { value: c.id, label: c.displayName };
  });
});
const componentsForSchemaVariantId = computed(() => {
  return componentsStore.allComponents.filter(
    (c) => c.schemaVariantId === props.schemaVariantId,
  );
});

const disableTestButton = computed(
  (): boolean =>
    !selectedComponentId.value ||
    !props.readyToTest ||
    (props.isAttributeFunc && !selectedPrototypeId.value),
);

const prototypeIsForSelectedComponent = (
  prototype: AttributePrototypeBag,
): boolean => {
  // First, we need to ensure the component and its asset have been selected . If not, we need to bail.
  if (!selectedComponentId.value) {
    return false;
  }

  // Default to checking if the prototype belongs to the component first. If it doesn't, we need to check
  // if it belongs its schema variant. If the prototype belongs to neither, it has been orphaned and we need to
  // error (this should not be possible, but we still want to check).
  if (prototype.componentId) {
    return prototype.componentId === selectedComponentId.value;
  } else if (prototype.schemaVariantId) {
    return prototype.schemaVariantId === props.schemaVariantId;
  } else {
    throw new Error(
      "prototype has been orphaned: it neither belongs to a component nor a schema variant",
    );
  }
};

// Only for attribute funcs, assemble prototypes that belong to the selected component.
const prototypeAttributeOptions = computed(
  (): {
    label: string;
    value: string;
  }[] => {
    const options = [];

    // Despite the fact that we can compile prototype options for _all_ components, we should wait until
    // the user selects a _single_ component.
    if (!funcStore.selectedFuncId || !selectedComponentId.value) return [];
    const selectedFunc = funcStore.selectedFuncDetails;

    if (selectedFunc?.associations?.type === "attribute") {
      const attributeAssociations = selectedFunc.associations;

      for (const prototype of attributeAssociations.prototypes) {
        if (prototypeIsForSelectedComponent(prototype)) {
          // Once we know the prototype belongs to either the selected component or its schema variant,
          // we assemble the option based on what output location the prototype is bound to. If it is
          // bound to nowhere, we need to error (this should not be possible, but we still want to check).
          if (prototype.propId) {
            options.push({
              label:
                funcStore.propIdToSourceName(prototype.propId) ??
                `Attribute: ${prototype.propId}`,
              value: prototype.id,
            });
          } else if (prototype.outputSocketId) {
            options.push({
              label:
                funcStore.outputSocketIdToSourceName(
                  prototype.outputSocketId,
                ) ?? `Output Socket: ${prototype.outputSocketId}`,
              value: prototype.id,
            });
          } else {
            throw new Error(
              "prototype to test is not bound to an output location",
            );
          }
        }
      }
      return options;
    } else {
      // If the selected func is not an attribute func, there are no options to assemble.
      return [];
    }
  },
);

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
