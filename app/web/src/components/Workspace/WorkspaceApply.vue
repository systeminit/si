<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <ResizablePanel rememberSizeKey="workflow-left" side="left" :minSize="315">
    <RecommendationPicker />
  </ResizablePanel>
  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <RecommendationProgressOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      :customConfig="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      readOnly
      deleteIcon="trash"
      @hover-element="onDiagramHoverElement"
      @update:selection="onDiagramUpdateSelection"
      @right-click-element="onRightClickElement"
    />
    <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
  </div>
  <ResizablePanel rememberSizeKey="workflow-right" side="right" :minSize="280">
    <TabGroup startSelectedTabSlug="apply-history">
      <TabGroupItem label="Apply History" slug="apply-history">
        <ApplyHistory />
      </TabGroupItem>
    </TabGroup>
  </ResizablePanel>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
  ResizablePanel,
  TabGroup,
  TabGroupItem,
} from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import RecommendationProgressOverlay from "@/components/RecommendationProgressOverlay.vue";
import RecommendationPicker from "../RecommendationPicker.vue";
import ApplyHistory from "../ApplyHistory.vue";
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

// TODO - any right click dropdown items we want for this diagram?
const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  return items;
});
</script>
