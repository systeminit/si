<template>
  <div class="flex flex-row w-full h-full">
    <!-- FIXME(nick): noticed that "whichComponent" has the following warning (but does not seem to matter right now) -->
    <!-- "Type {} is not assignable to type string | ComponentDefinition" -->
    <component
      :is="whichComponent"
      :is-visible="isVisible"
      :panel-index="panelIndex"
      :panel-ref="panelRef"
      :panel-container-ref="panelContainerRef"
      :initial-maximized-full="maximizedFull"
      :initial-maximized-container="maximizedContainer"
      :initial-panel-type="panelType"
      :kind="panelSubType"
      :is-maximized-container-enabled="isMaximizedContainerEnabled"
      @change-panel="changePanelType"
      @panel-maximize-full="setMaximizedFullTrue($event)"
      @panel-minimize-full="setMaximizedFullFalse($event)"
      @panel-maximize-container="setMaximizedContainerTrue($event)"
      @panel-minimize-container="setMaximizedContainerFalse($event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import { PanelMaximized, PanelType } from "./panel_types";
import PanelEmpty from "@/organisims/PanelEmpty.vue";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import { PanelAttributeSubType } from "./panel_types";

import PanelAttribute from "@/organisims/PanelAttribute.vue";
import PanelSchematic from "@/organisims/PanelSchematic.vue";
import PanelSecret from "@/organisims/PanelSecret.vue";

const props = defineProps<{
  panelIndex: number;
  panelRef: string;
  panelContainerRef: string;
  initialPanelType: PanelType;
  initialPanelSubType: SchematicKind | PanelAttributeSubType | null;
}>();
const emit = defineEmits([
  "change-panel",
  "panel-maximize-container",
  "panel-minimize-container",
  "panel-maximize-full",
  "panel-minimize-full",
]);
const panelSubType = ref<PanelAttributeSubType | SchematicKind | null>(
  props.initialPanelSubType,
);
const panelType = ref<PanelType>(props.initialPanelType);

const whichComponent = computed<
  | typeof PanelAttribute
  | typeof PanelSchematic
  | typeof PanelSecret
  | typeof PanelEmpty
>(() => {
  if (panelType.value == "attribute") {
    return PanelAttribute;
  } else if (panelType.value == "schematic") {
    return PanelSchematic;
  } else if (panelType.value == "secret") {
    return PanelSecret;
  } else {
    return PanelEmpty;
  }
});

const maximizedFull = ref<boolean>(false);
const maximizedContainer = ref<boolean>(false);
const isVisible = ref<boolean>(true);
const isMaximizedContainerEnabled = ref<boolean>(true);

const setMaximizedFullTrue = (event: PanelMaximized) => {
  maximizedFull.value = true;
  emit("panel-maximize-full", event);
};
const setMaximizedFullFalse = (event: PanelMaximized) => {
  maximizedFull.value = false;
  emit("panel-minimize-full", event);
};
const setMaximizedContainerTrue = (event: PanelMaximized) => {
  maximizedContainer.value = true;
  emit("panel-maximize-container", event);
};
const setMaximizedContainerFalse = (event: PanelMaximized) => {
  maximizedContainer.value = false;
  emit("panel-minimize-container", event);
};
const changePanelType = (newPanelType: PanelType) => {
  panelType.value = newPanelType;
  panelSubType.value = null;
  emit("change-panel", newPanelType);
};
</script>
