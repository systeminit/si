<template>
  <div class="flex flex-col w-full">
    <div class="flex flex-row items-center h-10 p-sm text-base align-middle">
      <div class="text-lg whitespace-nowrap overflow-hidden text-ellipsis">
        {{ lastSelectedComponent.schemaName }}
      </div>
      <!-- <div class="ml-2 flex" :aria-label="qualificationTooltip">
        <Icon name="check-circle" :class="qualificationColorClass" />
      </div>

      <div class="ml-2 flex" :aria-label="resourceTooltip">
        <Icon name="component" :class="resourceIconColorClass" />
      </div> -->

      <div
        class="flex flow-row items-center justify-end flex-grow h-full text-xs text-center"
      >
        <!-- <SiLink
          v-if="componentMetadata?.schemaLink"
          :uri="componentMetadata.schemaLink"
          blank-target
          class="m-2 flex"
        >
          <SiButtonIcon tooltip-text="Go to documentation" icon="help-circle" />
        </SiLink> -->

        <!-- <div
          v-if="editCount"
          class="flex flex-row items-center"
          aria-label="Number of edit fields"
        >
          <Icon name="edit" class="text-warning-600" />
          <div class="ml-1 text-center">{{ editCount }}</div>
        </div> -->
      </div>
    </div>

    <PropertyEditor
      v-if="editorContext"
      :editor-context="editorContext"
      @updated-property="updateProperty"
      @add-to-array="addToArray"
      @add-to-map="addToMap"
      @create-attribute-func="onCreateAttributeFunc"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";

import {
  UpdatedProperty,
  AddToArray,
  AddToMap,
  FuncWithPrototypeContext,
} from "@/api/sdf/dal/property_editor";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import { useRouteToFunc } from "@/utils/useRouteToFunc";
import { useComponentsStore } from "@/store/components.store";
import { useFuncStore } from "@/store/func/funcs.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import PropertyEditor from "./PropertyEditor.vue";

const funcStore = useFuncStore();
const routeToFunc = useRouteToFunc();

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
const getAttributeContext = (propId: number) => ({
  attribute_context_prop_id: propId,
  attribute_context_internal_provider_id: -1,
  attribute_context_external_provider_id: -1,
  attribute_context_schema_id: lastSelectedComponent.value.schemaId,
  attribute_context_schema_variant_id:
    lastSelectedComponent.value.schemaVariantId,
  attribute_context_component_id: lastSelectedComponent.value.id,
  attribute_context_system_id: -1,
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

const onCreateAttributeFunc = async (
  currentFunc: FuncWithPrototypeContext,
  valueId: number,
  parentValueId?: number,
) => {
  const res = await funcStore.CREATE_FUNC({
    kind: FuncBackendKind.JsAttribute,
    options: {
      valueId,
      parentValueId,
      componentId: lastSelectedComponent.value.id,
      schemaVariantId: lastSelectedComponent.value.schemaVariantId,
      schemaId: lastSelectedComponent.value.schemaId,
      currentFuncId: currentFunc.id,
      type: "attributeOptions",
    },
  });
  if (res.result.success) {
    routeToFunc(res.result.data.id);
  }
};
</script>
