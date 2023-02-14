<template>
  <div class="flex flex-col w-full">
    <PropertyEditor
      v-if="editorContext"
      :editor-context="editorContext"
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
import { useComponentsStore } from "@/store/components.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import PropertyEditor from "./PropertyEditor.vue";

const props = defineProps<{
  disabled?: boolean;
}>();

const componentsStore = useComponentsStore();
const componentId = computed(() => componentsStore.selectedComponentId);

const componentAttributesStore = useComponentAttributesStore();

const editorContext = computed(() => componentAttributesStore.editorContext);

const updateProperty = (event: UpdatedProperty) => {
  if (!componentId.value) return;

  const prop = editorContext.value?.schema.props[event.propId];

  if (prop?.name === "type") {
    componentAttributesStore.SET_COMPONENT_TYPE({
      value: event.value,
      componentId: componentId.value,
    });
  } else {
    componentAttributesStore.UPDATE_PROPERTY_VALUE({
      update: {
        attributeValueId: event.valueId,
        parentAttributeValueId: event.parentValueId,
        value: event.value,
        key: event.key,
        propId: event.propId,
        componentId: componentId.value,
      },
    });
  }
};

const addToArray = (event: AddToArray) => {
  if (!componentId.value) return;

  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      propId: event.propId,
      componentId: componentId.value,
    },
  });
};
const addToMap = (event: AddToMap) => {
  if (!componentId.value) return;
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      key: event.key,
      propId: event.propId,
      componentId: componentId.value,
    },
  });
};
</script>
