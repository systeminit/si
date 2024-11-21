<template>
  <div
    :class="
      clsx(
        'attributes-panel__si-settings',
        'flex flex-row items-center h-8 ml-md mr-xs my-xs',
      )
    "
  >
    <div
      :id="`color-picker-${componentId}`"
      ref="colorPickerMountRef"
      :style="{ backgroundColor: siValues.color }"
      :title="siValues.color"
      :class="
        clsx(
          'attributes-panel__color-swatch',
          'w-8 h-8 mr-xs shrink-0 cursor-pointer relative rounded border border-neutral-600',
          pickerOpen
            ? 'outline outline-2 outline-action-400 dark:outline-action-300'
            : 'hover:outline hover:outline-2 hover:outline-action-400 dark:hover:outline-action-300',
        )
      "
      @click="openColorPicker"
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
    <IconButton
      class="flex-none ml-2xs"
      iconTone="action"
      :icon="COMPONENT_TYPE_ICONS[siValues.type]"
      :tooltip="
        {
          component: 'Component',
          configurationFrameUp: 'Up Frame',
          configurationFrameDown: 'Down Frame',
          aggregationFrame: 'Frame',
        }[siValues.type]
      "
      size="lg"
      tooltipPlacement="top"
      :selected="typeMenuRef?.isOpen"
      @click="openTypeMenu"
    >
      <DropdownMenu ref="typeMenuRef" forceAlignRight>
        <DropdownMenuItem
          icon="component"
          label="Component"
          checkable
          :checked="siValues.type === 'component'"
          @select="updateComponentType(ComponentType.Component)"
        />
        <DropdownMenuItem
          icon="frame-up"
          label="Up Frame"
          checkable
          :checked="siValues.type === 'configurationFrameUp'"
          @select="updateComponentType(ComponentType.ConfigurationFrameUp)"
        />
        <DropdownMenuItem
          icon="frame-down"
          label="Down Frame"
          checkable
          :checked="siValues.type === 'configurationFrameDown'"
          @select="updateComponentType(ComponentType.ConfigurationFrameDown)"
        />
      </DropdownMenu>
    </IconButton>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed, reactive, ref, watch } from "vue";
import * as _ from "lodash-es";
import Picker from "vanilla-picker";
import {
  COMPONENT_TYPE_ICONS,
  DropdownMenu,
  DropdownMenuItem,
  IconButton,
  themeClasses,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";
import { ComponentType } from "@/api/sdf/dal/schema";

const componentsStore = useComponentsStore();
const componentId = componentsStore.selectedComponent?.def.id;
if (!componentId) {
  throw new Error("Do not use this component without a selectedComponentId");
}
const component = componentsStore.selectedComponent;

const attributesStore = useComponentAttributesStore(componentId || "NONE");

// Special handling of SI part of the tree (name, color, etc) /////////////////////////////////////////////////

const siProps = computed(() => attributesStore.siTreeByPropName);

// we have the component info from the loaded component already, but we are ideally grabbing it from the attributes tree
// in case in the future we may want to show more info (like where the value is coming from, its update status, etc...)
const siValuesFromStore = computed(() => ({
  name:
    (siProps.value?.name?.value?.value as string) || component.def.displayName,
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

function updateComponentType(type = siValues.type) {
  siValues.type = type;
  attributesStore.SET_COMPONENT_TYPE({
    componentId: component.def.id,
    componentType: siValues.type,
  });
}

const typeMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const openTypeMenu = (e: MouseEvent) => {
  typeMenuRef.value?.open(e);
};

// color picker
const colorPickerMountRef = ref<HTMLElement>();
const pickerOpen = ref(false);
let picker: Picker | undefined;
function openColorPicker() {
  if (!picker) {
    picker = new Picker({
      parent: colorPickerMountRef.value,
      alpha: false,
      color: siValues.color,
      onDone(color: { hex: string }) {
        siValues.color = color.hex.substring(0, color.hex.length - 2);
        updateSiProp("color");
        picker?.destroy();
        picker = undefined;
      },
    });
    picker.onClose = () => {
      pickerOpen.value = false;
    };
  }
  picker.show();
  pickerOpen.value = true;
}
</script>
