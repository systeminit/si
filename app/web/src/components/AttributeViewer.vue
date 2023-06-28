<template>
  <div class="flex flex-col w-full">
    <ReadOnlyBanner v-if="disabled" class="border-b-2" />
    <PropertyEditor
      v-if="editorContext"
      :editorContext="editorContext"
      :disabled="props.disabled"
      @updated-property="updateProperty"
      @add-to-array="addToArray"
      @add-to-map="addToMap"
    />
    <div v-else class="p-md text-center text-lg">Loading...</div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import {
  UpdatedProperty,
  AddToArray,
  AddToMap,
} from "@/api/sdf/dal/property_editor";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { useComponentsStore } from "@/store/components.store";
import PropertyEditor from "./PropertyEditor.vue";
import ReadOnlyBanner from "./ReadOnlyBanner.vue";

const props = defineProps<{
  disabled?: boolean;
}>();

// NON-REACTIVE component id. This works because the parent has a :key which rerenders if the selected component changes
const componentsStore = useComponentsStore();
const componentId = componentsStore.selectedComponentId;
if (!componentId) {
  throw new Error("Do not use this component without a selectedComponentId");
}

const componentAttributesStore = useComponentAttributesStore(componentId);
const editorContext = computed(() => componentAttributesStore.editorContext);

const updateProperty = (event: UpdatedProperty) => {
  const prop = editorContext.value?.schema.props[event.propId];

  if (prop?.name === "type") {
    componentAttributesStore.SET_COMPONENT_TYPE({
      value: event.value,
      componentId,
    });
  } else {
    componentAttributesStore.UPDATE_PROPERTY_VALUE({
      update: {
        attributeValueId: event.valueId,
        parentAttributeValueId: event.parentValueId,
        value: event.value,
        key: event.key,
        propId: event.propId,
        componentId,
      },
    });
  }
};

const addToArray = (event: AddToArray) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      propId: event.propId,
      componentId,
    },
  });
};
const addToMap = (event: AddToMap) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      key: event.key,
      propId: event.propId,
      componentId,
    },
  });
};
</script>
