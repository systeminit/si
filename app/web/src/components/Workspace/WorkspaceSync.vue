<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="workflow-left" side="left" :min-size="315">
    <ComponentOutlineSync />
  </SiPanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <RecommendationProgressOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      :custom-config="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      read-only
      delete-icon="trash"
      @hover-element="onDiagramHoverElement"
      @update:selection="onDiagramUpdateSelection"
      @right-click-element="onRightClickElement"
    />
    <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
  </div>
  <SiPanel remember-size-key="workflow-right" side="right" :min-size="280">
    <SyncExecutor />
  </SiPanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import RecommendationProgressOverlay from "@/components/RecommendationProgressOverlay.vue";
import SiPanel from "@/components/SiPanel.vue";
import SyncExecutor from "@/components/SyncExecutor.vue";
import ComponentOutlineSync from "../ComponentOutlineSync.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import {
  DiagramEdgeData,
  DiagramGroupData,
  DiagramNodeData,
  HoverElementEvent,
  RightClickElementEvent,
  SelectElementEvent,
} from "../GenericDiagram/diagram_types";

const componentsStore = useComponentsStore();
const diagramNodes = computed(() => componentsStore.diagramNodes);
const diagramEdges = computed(() => componentsStore.diagramEdges);

const diagramCustomConfig = {};

// HOVER
function onDiagramHoverElement(newHover: HoverElementEvent) {
  if (
    newHover.element instanceof DiagramNodeData ||
    newHover.element instanceof DiagramGroupData
  ) {
    componentsStore.setHoveredComponentId(newHover.element.def.componentId);
  } else if (newHover.element instanceof DiagramEdgeData) {
    componentsStore.setHoveredEdgeId(newHover.element.def.id);
  } else {
    // handles case of hovering nothing and hovering edges
    componentsStore.setHoveredComponentId(null);
  }
}

function onDiagramUpdateSelection(newSelection: SelectElementEvent) {
  if (
    newSelection.elements.length === 1 &&
    newSelection.elements[0] instanceof DiagramEdgeData
  ) {
    componentsStore.setSelectedEdgeId(newSelection.elements[0].def.id);
  } else {
    const validComponentIds = _.compact(
      newSelection.elements.map((el) => {
        if (el instanceof DiagramNodeData || el instanceof DiagramGroupData) {
          return el.def.componentId;
        }
        return undefined;
      }),
    );
    componentsStore.setSelectedComponentId(validComponentIds);
  }
}

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  contextMenuRef.value?.open(rightClickEventInfo.e, true);
}

// TODO - Add "Alter Simulation" option
const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  return items;
});
</script>
