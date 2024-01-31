<template>
  <div
    ref="rootRef"
    class="attributes-panel"
    :class="{
      '--show-section-toggles': showSectionToggles,
    }"
    @pointermove="onMouseMove"
    @mouseleave="onMouseLeave"
  >
    <!-- custom inputs for SI props (name, color, etc) -->
    <div class="attributes-panel__si-settings">
      <div
        :id="`color-picker-${componentId}`"
        ref="colorPickerMountRef"
        :title="siValues.color"
        class="attributes-panel__color-swatch"
        :style="{ backgroundColor: siValues.color }"
        @click="openColorPicker"
      />
      <input
        v-model="siValues.name"
        class="attributes-panel__name-input"
        type="text"
        @blur="updateSiProp('name')"
        @keyup.enter="updateSiProp('name')"
      />
      <div class="attributes-panel__type-dropdown">
        <select v-model="siValues.type" @change="updateNodeType()">
          <option value="component">Component</option>
          <option value="configurationFrameUp">Configuration Frame (Up)</option>
          <option value="configurationFrameDown">
            Configuration Frame (Down)
          </option>
        </select>
        <Icon name="chevron--down" />
      </div>
    </div>

    <LoadingMessage v-if="loadSchemaReqStatus.isPending && !domainTree">
      Loading asset schema
    </LoadingMessage>
    <div v-else-if="domainTree" class="attributes-panel__items-wrap">
      <AttributesPanelItem
        v-if="domainTree && domainTree.children.length"
        :attributeDef="domainTree"
      />
      <AttributesPanelItem
        v-if="secretsTree && secretsTree.children.length"
        :attributeDef="secretsTree"
      />
    </div>

    <div v-if="SHOW_DEBUG_TREE" class="mt-xl">
      <JsonTreeExplorer :object="domainTree" />
    </div>

    <!-- todo - show this when right clicking attributes -->
    <DropdownMenu ref="contextMenuRef">
      <DropdownMenuItem icon="dots-horizontal" label="Copy JSON path" />
    </DropdownMenu>
  </div>
</template>

<script lang="ts">
type EventBusEvents = { toggleAllOpen: boolean };

type AttributesPanelContext = {
  openContextMenu(e: MouseEvent, path: string): void;
  eventBus: Emitter<EventBusEvents>;
  hoverSectionValueId: Ref<string | undefined>;
  showSectionToggles: Ref<boolean>;
};

export const AttributesPanelContextInjectionKey: InjectionKey<AttributesPanelContext> =
  Symbol("AttributesPanelContext");

export function useAttributesPanelContext() {
  const ctx = inject(AttributesPanelContextInjectionKey, null);
  if (!ctx)
    throw new Error(
      "<AttributesPanelItem> should only be used within a <AttributesPanel>",
    );
  return ctx;
}
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script setup lang="ts">
import Picker from "vanilla-picker";
import * as _ from "lodash-es";
import {
  InjectionKey,
  Ref,
  computed,
  inject,
  provide,
  reactive,
  ref,
  watch,
} from "vue";
import mitt, { Emitter } from "mitt";

import {
  DropdownMenu,
  DropdownMenuItem,
  JsonTreeExplorer,
  LoadingMessage,
  Icon,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";

import AttributesPanelItem from "./AttributesPanelItem.vue";

const props = defineProps({
  object: Object,
  numPreviewProps: { type: Number, default: 3 },
});

const rootRef = ref<HTMLDivElement>();

// toggle to true to show JSON tree explorer or attributes/values
const SHOW_DEBUG_TREE = false;

// NON-REACTIVE component id. This works because the parent has a :key which rerenders if the selected component changes
const componentsStore = useComponentsStore();
const componentId = componentsStore.selectedComponent?.id;
if (!componentId) {
  throw new Error("Do not use this component without a selectedComponentId");
}
const component = componentsStore.selectedComponent;

const attributesStore = useComponentAttributesStore(componentId || "NONE");

const loadSchemaReqStatus = attributesStore.getRequestStatus(
  "FETCH_PROPERTY_EDITOR_SCHEMA",
);

// Special handling of SI part of the tree (name, color, etc) /////////////////////////////////////////////////

const siProps = computed(() => attributesStore.siTreeByPropName);

// we have the component info from the loaded component already, but we are ideally grabbing it from the attributes tree
// in case in the future we may want to show more info (like where the value is coming from, its update status, etc...)
const siValuesFromStore = computed(() => ({
  name: (siProps.value?.name?.value?.value as string) || component.displayName,
  color: (siProps.value?.color?.value?.value as string) || component.color,
  type: (siProps.value?.type?.value?.value as string) || component?.nodeType,
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
      componentId: component.id,
      value: newVal,
    },
  });
}

// color picker
const colorPickerMountRef = ref<HTMLElement>();
function updateNodeType() {
  attributesStore.SET_COMPONENT_TYPE({
    componentId: component.id,
    value: siValues.type,
  });
}

let picker: Picker | undefined;
function openColorPicker() {
  if (!picker) {
    picker = new Picker({
      parent: colorPickerMountRef.value,
      alpha: false,
      onDone(color: { hex: string }) {
        siValues.color = color.hex.substring(0, color.hex.length - 2);
        updateSiProp("color");
        picker?.destroy();
        picker = undefined;
      },
    });
  }
  picker.show();
}

const domainTree = computed(() => attributesStore.domainTree);
const secretsTree = computed(() => attributesStore.secretsTree);

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const contextMenuPath = ref<string>();

function openContextMenu(e: MouseEvent, path: string) {
  contextMenuRef.value?.open(e, true);
  contextMenuPath.value = path;
}

// function toggleAllOpen(open: boolean) {
//   eventBus.emit("toggleAllOpen", open);
// }

const showSectionToggles = ref(false);
function onMouseMove(e: PointerEvent) {
  const rect = rootRef.value?.getBoundingClientRect();
  if (!rect) return;
  const x = e.clientX - rect.left; // x position within the root div
  showSectionToggles.value = x >= 0 && x <= 24;
}
function onMouseLeave() {
  showSectionToggles.value = false;
}

// EXPOSED TO CHILDREN
const eventBus = mitt<EventBusEvents>();
const hoverSectionValueId = ref<string>();

provide(AttributesPanelContextInjectionKey, {
  openContextMenu,
  eventBus,
  hoverSectionValueId,
  showSectionToggles,
});
</script>

<style lang="less">
.attributes-panel {
  display: flex;
  flex-direction: column;
  min-height: 100%;
  body.light & {
    --toggle-controls-bg-color: @colors-neutral-100;
  }
  body.dark & {
    --toggle-controls-bg-color: @colors-neutral-900;
  }
}

.attributes-panel__items-wrap {
  position: relative;
  padding-bottom: @spacing-px[md];
  flex-grow: 1;

  // darker bg behind section collapse toggles (left)
  &:before {
    content: "";
    position: absolute;
    left: 0;
    width: 24px;
    top: 0;
    bottom: 0;
    background: var(--toggle-controls-bg-color);
  }
}

.attributes-panel__color-swatch {
  width: 40px;
  height: 40px;
  border: 1px solid var(--input-border-color);
  margin-right: @spacing-px[xs];
  flex-shrink: 0;
  cursor: pointer;
  position: relative;

  .picker_wrapper.popup,
  .picker_wrapper.popup .picker_arrow::before,
  .picker_wrapper.popup .picker_arrow::after {
    background: white;
    z-index: 500;
    body.dark & {
      background: black;
    }
  }
}

.attributes-panel__si-settings {
  display: flex;
  height: 40px;
  margin: @spacing-px[sm];
  margin-left: @spacing-px[md];
  margin-right: 8px;

  input,
  select {
    @apply focus:ring-0 focus:ring-offset-0;
    display: block;
    height: inherit;
    position: relative;
    background: transparent;
    border: 1px solid var(--input-border-color);
    background: var(--input-bg-color);
    margin-left: -1px;
    font-size: 16px;
    padding: 2px 8px;

    &:focus {
      background: var(--input-focus-bg-color);
      border-color: var(--input-focus-border-color);

      z-index: 2;
      border-color: var(--input-focus-border-color);
    }
  }
}
.attributes-panel__name-input {
  flex-grow: 1;
  flex-shrink: 1;
  min-width: 50px;
}
.attributes-panel__type-dropdown {
  position: relative;
  height: inherit;
  flex-grow: 1;
  min-width: 120px;
  select {
    position: absolute;
    width: 100%;
    height: 100%;
    padding-right: 30px;
    text-overflow: ellipsis;
  }
  .icon {
    position: absolute;
    right: 4px;
    top: 0;
    height: inherit;
    z-index: 3;
  }
}
</style>
