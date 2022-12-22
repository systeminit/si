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

function nilId(): string {
  return "00000000000000000000000000";
}

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

// TODO: not sure why we need to pass this all back to the backend - seems like we should pass the minimal data
const getAttributeContext = (propId: string) => ({
  attribute_context_prop_id: propId,
  attribute_context_internal_provider_id: nilId(),
  attribute_context_external_provider_id: nilId(),
  attribute_context_component_id: lastSelectedComponent.value.id,
});

const updateProperty = (event: UpdatedProperty) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    update: {
      attributeValueId: event.valueId,
      parentAttributeValueId: event.parentValueId,
      value: event.value,
      key: event.key,
      attributeContext: getAttributeContext(event.propId),
    },
  });
};

const addToArray = (event: AddToArray) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      attributeContext: getAttributeContext(event.propId),
    },
  });
};
const addToMap = (event: AddToMap) => {
  componentAttributesStore.UPDATE_PROPERTY_VALUE({
    insert: {
      parentAttributeValueId: event.valueId,
      key: event.key,
      attributeContext: getAttributeContext(event.propId),
    },
  });
};
</script>
