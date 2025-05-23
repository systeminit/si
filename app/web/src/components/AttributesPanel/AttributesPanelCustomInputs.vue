<template>
  <div
    :class="
      clsx(
        'attributes-panel__si-settings',
        'flex flex-row gap-2xs items-center h-8 m-xs',
      )
    "
  >
    <ColorPicker
      id="attributes-panel-component-color-picker"
      v-model="siValues.color"
      required
      variant="box"
      class="flex-none"
      @change="updateColor"
    />
    <input
      v-model="siValues.name"
      :class="
        clsx(
          'attributes-panel__name-input',
          'block relative grow shrink min-w-[50px] h-8 px-xs py-3xs text-md',
          'border focus:z-10',
          themeClasses(
            'border-neutral-400 bg-neutral-100 focus:border-action-500 focus:bg-shade-0',
            'border-neutral-600 bg-neutral-900 focus:border-action-300 focus:bg-shade-100',
          ),
        )
      "
      type="text"
      @blur="updateSiProp('name')"
      @keyup.enter="updateSiProp('name')"
    />
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed, reactive, watch } from "vue";
import * as _ from "lodash-es";
import { themeClasses, ColorPicker } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useViewsStore } from "@/store/views.store";
import { DiagramViewData } from "../ModelingDiagram/diagram_types";

const viewStore = useViewsStore();
const componentsStore = useComponentsStore();
const componentId = viewStore.selectedComponent?.def.id;
if (!componentId) {
  throw new Error("Do not use this component without a selectedComponentId");
}
const component = viewStore.selectedComponent;

const attributesStore = useComponentAttributesStore(componentId || "NONE");

// Special handling of SI part of the tree (name, color, etc) /////////////////////////////////////////////////

const siProps = computed(() => attributesStore.siTreeByPropName);

// we have the component info from the loaded component already, but we are ideally grabbing it from the attributes tree
// in case in the future we may want to show more info (like where the value is coming from, its update status, etc...)
const siValuesFromStore = computed(() => ({
  name:
    (siProps.value?.name?.value?.value as string) ||
    ("displayName" in component.def && component.def.displayName) ||
    ("name" in component.def && component.def.name) ||
    "",
  color: (siProps.value?.color?.value?.value as string) || component.def.color,
  type:
    (siProps.value?.type?.value?.value as ComponentType) ||
    component?.def.componentType,
}));
const siValues = reactive(_.cloneDeep(siValuesFromStore.value));

watch(
  siValuesFromStore,
  (newVal, oldVal) => {
    // as the schema and validations are reloaded, the watcher fires multiple times
    // but what we actually care about is if the values themselves have truly changed
    if (!_.isEqual(newVal, oldVal)) {
      _.assign(siValues, siValuesFromStore.value);
    }
  },
  { deep: true },
);
function updateSiProp(key: keyof typeof siValues) {
  if (component instanceof DiagramViewData) return;
  if (key === "name") siValues[key] = siValues[key].trim();

  const newVal = siValues[key];
  if (newVal === siValuesFromStore.value[key]) return;

  const prop = siProps.value?.[key as string];
  if (!prop) return;

  attributesStore.UPDATE_PROPERTY_VALUE({
    update: {
      attributeValueId: prop.valueId,
      parentAttributeValueId: prop.parentValueId,
      propId: prop.propId,
      componentId: component.def.id,
      value: newVal,
      isForSecret: false,
    },
  });
  if (key === "name" && newVal) {
    // TODO; after DVU completes, backend should send updated component object models over WsEvent
    componentsStore.setComponentDisplayName(component, newVal);
  }
}

const updateColor = (hexColor: string) => {
  siValues.color = hexColor;
  updateSiProp("color");
};
</script>
