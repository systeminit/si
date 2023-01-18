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
// Note(victor): This component will only be rendered if there's a selected component.
// To avoid weird data races where the store has already unset the value but we still need to use it, we can default to
// using lastSelectedComponent instead of selectedComponent.
// This helps us, for example, to save attributes onBeforeUnmount here or on any children.
const lastSelectedComponent = computed(
  () => componentsStore.lastSelectedComponent,
);

const componentAttributesStore = useComponentAttributesStore();

const editorContext = computed(() => componentAttributesStore.editorContext);

const updateProperty = (event: UpdatedProperty) => {
  const prop = editorContext.value?.schema.props[event.propId];

  if (prop?.name === "type") {
    componentAttributesStore.SET_COMPONENT_TYPE({
      value: event.value,
      componentId: lastSelectedComponent.value.id,
    });
  } else {
    componentAttributesStore.UPDATE_PROPERTY_VALUE({
      update: {
        attributeValueId: event.valueId,
        parentAttributeValueId: event.parentValueId,
        value: event.value,
        key: event.key,
        propId: event.propId,
        componentId: lastSelectedComponent.value.id,
      },
    });
  }
};

const addToArray = (event: AddToArray) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      propId: event.propId,
      componentId: lastSelectedComponent.value.id,
    },
  });
};
const addToMap = (event: AddToMap) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      key: event.key,
      propId: event.propId,
      componentId: lastSelectedComponent.value.id,
    },
  });
};
</script>
