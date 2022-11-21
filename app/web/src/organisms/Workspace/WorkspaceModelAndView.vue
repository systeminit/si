<template>
  <SiPanel remember-size-key="changeset-and-asset" side="left" :min-size="250">
    <div class="flex flex-col h-full">
      <ChangeSetPanel
        v-if="!isViewMode"
        ref="changeSetPanelRef"
        class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0"
      />

      <SiTabGroup class="relative flex-grow">
        <template #tabs>
          <SiTabHeader v-if="!isViewMode">Asset Palette</SiTabHeader>
          <SiTabHeader>Diagram Outline</SiTabHeader>
        </template>

        <template #panels>
          <TabPanel v-if="!isViewMode">
            <AssetPalette />
          </TabPanel>
          <TabPanel>
            <DiagramOutline
              :selected-component-id="selectedComponentId ?? undefined"
              @select="onOutlineSelectComponent"
            />
          </TabPanel>
        </template>
      </SiTabGroup>
    </div>
  </SiPanel>

  <div class="grow h-full relative bg-neutral-50 dark:bg-neutral-900">
    <GlobalStatusOverlay />
    <GenericDiagram
      v-if="diagramNodes"
      ref="diagramRef"
      :custom-config="diagramCustomConfig"
      :nodes="diagramNodes"
      :edges="diagramEdges"
      :read-only="isViewMode"
      :controls-disabled="changeSetPanelRef?.showDialog === undefined"
      @insert-element="onDiagramInsertElement"
      @move-element="onDiagramMoveElement"
      @draw-edge="onDrawEdge"
      @delete-elements="onDiagramDelete"
      @update:selection="onDiagramUpdateSelection"
      @right-click-element="onRightClickElement"
    />
    <DropdownMenu ref="contextMenuRef">
      <DropdownMenuItem icon="trash">Delete component</DropdownMenuItem>
    </DropdownMenu>
  </div>

  <SiPanel
    remember-size-key="component-details"
    side="right"
    :default-size="380"
    :min-size="300"
  >
    <ComponentDetails v-if="selectedComponent" :key="selectedComponent.id" />
    <div v-else class="p-4">
      <template v-if="isViewMode">
        Select a single component to see more details
      </template>
      <template v-else>Select a single component to edit it </template>
    </div>
  </SiPanel>
</template>

<script lang="ts" setup>
import { TabPanel } from "@headlessui/vue";
import _ from "lodash";
import { computed, ref, watch } from "vue";
import { useRoute } from "vue-router";
import SiPanel from "@/atoms/SiPanel.vue";
import ChangeSetPanel from "@/organisms/ChangeSetPanel.vue";
import ComponentDetails from "@/organisms/ComponentDetails.vue";
import SiTabGroup from "@/molecules/SiTabGroup.vue";
import SiTabHeader from "@/molecules/SiTabHeader.vue";
import { useComponentsStore } from "@/store/components.store";
import DropdownMenu from "@/ui-lib/menus/DropdownMenu.vue";
import DropdownMenuItem from "@/ui-lib/menus/DropdownMenuItem.vue";
import { useStatusStore } from "@/store/status.store";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import {
  InsertElementEvent,
  MoveElementEvent,
  DrawEdgeEvent,
  DiagramElementIdentifier,
  DeleteElementsEvent,
  RightClickElementEvent,
} from "../GenericDiagram/diagram_types";
import DiagramOutline from "../DiagramOutline.vue";
import GlobalStatusOverlay from "../GlobalStatusOverlay.vue";

const currentRoute = useRoute();

// TODO: we'll very likely split view mode from compose mode again, so this is just temporary
// but for now we watch if the route is for view mode, and if so, switch to head and toggle a few things
const isViewMode = computed(() => currentRoute.name === "workspace-view");

const diagramRef = ref<InstanceType<typeof GenericDiagram>>();
const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const componentsStore = useComponentsStore();
// TODO: probably want to get more generic component data and then transform into diagram nodes
const diagramEdges = computed(() => componentsStore.diagramEdges);
const diagramNodes = computed(() => componentsStore.diagramNodes);

const selectedComponentId = computed(() => componentsStore.selectedComponentId);

const diagramCustomConfig = {};

const selectedComponent = computed(() => componentsStore.selectedComponent);

const insertCallbacks: Record<string, () => void> = {};

const changeSetPanelRef = ref();

watch(
  () => componentsStore.selectedInsertSchemaId,
  () => {
    if (componentsStore.selectedInsertSchemaId) {
      diagramRef.value?.beginInsertElement("node");
    } else {
      diagramRef.value?.endInsertElement();
    }
  },
);

watch([diagramNodes, diagramEdges], () => {
  // TODO: this should be firing off the callback only when we find the matching new node, but we dont have the new ID yet
  _.each(insertCallbacks, (insertCallback, newNodeId) => {
    insertCallback();
    delete insertCallbacks[newNodeId];
  });
});

async function onDrawEdge(e: DrawEdgeEvent) {
  const [fromNodeId, fromSocketId] = e.fromSocketId.split("-");
  const [toNodeId, toSocketId] = e.toSocketId.split("-");

  await componentsStore.CREATE_COMPONENT_CONNECTION(
    { componentId: parseInt(fromNodeId), socketId: parseInt(fromSocketId) },
    { componentId: parseInt(toNodeId), socketId: parseInt(toSocketId) },
  );
}

async function onDiagramInsertElement(e: InsertElementEvent) {
  if (!componentsStore.selectedInsertSchemaId)
    throw new Error("missing insert selection metadata");

  const schemaId = componentsStore.selectedInsertSchemaId;
  componentsStore.selectedInsertSchemaId = null;
  await componentsStore.CREATE_COMPONENT(schemaId, e.position);

  // TODO: we actually want the new node ID so we can watch for it in the updated data
  // but the API currently doesn't have it right away :(
  const newNodeId = +new Date();
  insertCallbacks[newNodeId] = e.onComplete;
}

function onDiagramMoveElement(e: MoveElementEvent) {
  // this gets called many times during a move, with e.isFinal telling you if the drag is in progress or complete
  // eventually we will want to send those to the backend for realtime multiplayer
  // But for now we just send off the final position
  if (!e.isFinal) return;
  componentsStore.SET_COMPONENT_DIAGRAM_POSITION(parseInt(e.id), e.position);
}

function onDiagramUpdateSelection(newSelection: DiagramElementIdentifier[]) {
  // for now, we dont support multiselect anywhere outside the diagram, so we just act like nothing is selected
  if (newSelection.length !== 1) {
    componentsStore.setSelectedComponentId(null);
    return;
  }

  const selectedElement = newSelection[0];
  // we also dont support selecting things other than nodes outside the diagram
  if (selectedElement.diagramElementType !== "node") {
    componentsStore.setSelectedComponentId(null);
    return;
  }
  componentsStore.setSelectedComponentId(parseInt(selectedElement.id));
}

function onDiagramDelete(_e: DeleteElementsEvent) {
  // eslint-disable-next-line no-alert
  alert("Deletion not supported yet!");
}

function onOutlineSelectComponent(id: number) {
  componentsStore.setSelectedComponentId(id);
}

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  contextMenuRef.value?.open(rightClickEventInfo.e, true);
}

watch(
  () => selectedComponentId.value,
  () => {
    if (selectedComponentId.value) {
      diagramRef.value?.setSelection({
        diagramElementType: "node",
        id: selectedComponentId.value.toString(),
      });
    } else {
      diagramRef.value?.clearSelection();
    }
  },
);

const statusStore = useStatusStore();
</script>
