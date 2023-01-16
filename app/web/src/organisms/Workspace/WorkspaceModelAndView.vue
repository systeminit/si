<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <SiPanel remember-size-key="changeset-and-asset" side="left" :min-size="250">
    <div class="flex flex-col h-full">
      <ChangeSetPanel v-if="!isViewMode" />

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
              @multiselect="onOutlineMultiSelectComponent"
              @pan="onOutlinePanToComponent"
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
      @insert-element="onDiagramInsertElement"
      @move-element="onDiagramMoveElement"
      @resize-element="onDiagramResizeElement"
      @group-elements="onGroupElements"
      @draw-edge="onDrawEdge"
      @delete-elements="onDiagramDelete"
      @update:selection="onDiagramUpdateSelection"
      @right-click-element="onRightClickElement"
    />

    <DropdownMenu ref="contextMenuRef">
      <template v-if="selectedEdgeId">
        <DropdownMenuItem icon="trash" @select="triggerDeleteSelection">
          Delete edge
        </DropdownMenuItem>
      </template>
      <template v-else-if="selectedComponentId">
        <DropdownMenuItem icon="trash" @select="triggerDeleteSelection">
          Delete component
        </DropdownMenuItem>
      </template>
      <template v-else>
        <DropdownMenuItem icon="help-circle" disabled>
          empty menu?
        </DropdownMenuItem>
      </template>
    </DropdownMenu>
  </div>

  <SiPanel
    remember-size-key="component-details"
    side="right"
    :default-size="380"
    :min-size="300"
  >
    <ComponentDetails
      v-if="selectedComponent"
      :key="selectedComponent.id"
      :disabled="isViewMode"
    />
    <div v-else class="p-4">
      <template v-if="isViewMode">
        Select a single component to see more details
      </template>
      <template v-else>Select a single component to edit it</template>
    </div>
  </SiPanel>

  <Modal ref="confirmDeleteModalRef" title="Are you sure?">
    <Stack space="sm">
      <p>You're about to delete some things.</p>
      <p>
        These items will be marked for deletion in this change set. When this
        change set is merged, they will be removed from your model.
      </p>

      <div class="flex space-x-sm justify-end">
        <VButton2
          icon="x"
          tone="shade"
          variant="ghost"
          @click="confirmDeleteModalRef?.close()"
        >
          Cancel
        </VButton2>
        <VButton2 icon="trash" tone="destructive" @click="onConfirmDelete">
          Confirm
        </VButton2>
      </div>
    </Stack>
  </Modal>
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
import Modal from "@/ui-lib/modals/Modal.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import GenericDiagram from "../GenericDiagram/GenericDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import {
  InsertElementEvent,
  MoveElementEvent,
  DrawEdgeEvent,
  DeleteElementsEvent,
  RightClickElementEvent,
  DiagramNodeData,
  DiagramGroupData,
  GroupEvent,
  SelectElementEvent,
  ResizeElementEvent,
  DiagramEdgeData,
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
const diagramEdges = computed(() => {
  // Note(victor): The code below checks whether was only created implicitly, through inheritance from an aggregation frame
  // In the future, it would make more sense for this to be stored on the database
  const edges = _.map(componentsStore.diagramEdges, (edge) => {
    edge.isInvisible = false;

    const toNodeParentId =
      componentsStore.componentsByNodeId[edge.toNodeId].parentId;

    if (toNodeParentId) {
      const toNodeParentComp =
        componentsStore.componentsByNodeId[toNodeParentId];

      if (toNodeParentComp.nodeType === "aggregationFrame") {
        if (edge.fromNodeId === toNodeParentComp.nodeId) {
          edge.isInvisible ||= true;
        }
      }
    }

    const fromNodeParentId =
      componentsStore.componentsByNodeId[edge.fromNodeId].parentId;

    if (fromNodeParentId) {
      const fromParentComp =
        componentsStore.componentsByNodeId[fromNodeParentId];
      if (fromParentComp.nodeType === "aggregationFrame") {
        if (edge.toNodeId === fromParentComp.nodeId) {
          edge.isInvisible ||= true;
        }
      }
    }

    return edge;
  });

  return edges;
});
const diagramNodes = computed(() => componentsStore.diagramNodes);

const selectedComponentId = computed(() => componentsStore.selectedComponentId);
const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);
const selectedEdgeId = computed(() => componentsStore.selectedEdgeId);

const diagramCustomConfig = {};

const selectedComponent = computed(() => componentsStore.selectedComponent);

const insertCallbacks: Record<string, () => void> = {};

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
  await componentsStore.CREATE_COMPONENT_CONNECTION(
    {
      nodeId: e.fromSocket.parent.def.id,
      socketId: e.fromSocket.def.id,
    },
    {
      nodeId: e.toSocket.parent.def.id,
      socketId: e.toSocket.def.id,
    },
  );
}

async function onDiagramInsertElement(e: InsertElementEvent) {
  if (!componentsStore.selectedInsertSchemaId)
    throw new Error("missing insert selection metadata");

  const schemaId = componentsStore.selectedInsertSchemaId;
  componentsStore.selectedInsertSchemaId = null;

  let parentId;

  if (e.parent) {
    const parentComponent = Object.values(componentsStore.componentsById).find(
      (c) => c.nodeId === e.parent,
    );
    if (
      parentComponent &&
      (parentComponent.nodeType !== "aggregationFrame" ||
        schemaId === parentComponent.schemaId)
    ) {
      parentId = e.parent;
    }
  }

  // TODO These ids should be number from the start.
  await componentsStore.CREATE_COMPONENT(schemaId, e.position, parentId);

  // TODO: we actually want the new node ID so we can watch for it in the updated data
  // but the API currently doesn't have it right away :(
  const newNodeId = +new Date();
  insertCallbacks[newNodeId] = e.onComplete;
}

function onDiagramResizeElement(e: ResizeElementEvent) {
  if (!e.isFinal) return;
  if (e.element instanceof DiagramGroupData) {
    componentsStore.SET_COMPONENT_DIAGRAM_POSITION(
      e.element.def.id,
      e.position,
      e.size,
    );
  }
}

function onDiagramMoveElement(e: MoveElementEvent) {
  // this gets called many times during a move, with e.isFinal telling you if the drag is in progress or complete
  // eventually we will want to send those to the backend for realtime multiplayer
  // But for now we just send off the final position
  if (!e.isFinal) return;
  if (
    e.element instanceof DiagramNodeData ||
    e.element instanceof DiagramGroupData
  ) {
    componentsStore.SET_COMPONENT_DIAGRAM_POSITION(
      e.element.def.id,
      e.position,
    );
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

const confirmDeleteModalRef = ref<InstanceType<typeof Modal>>();

function onDiagramDelete(_e: DeleteElementsEvent) {
  // delete event includes what to delete, but its the same as current selection
  triggerDeleteSelection();
}

function onOutlineSelectComponent(id: string) {
  componentsStore.setSelectedComponentId(id);
}

function onOutlineMultiSelectComponent(id: string) {
  const selectedComponentIds = componentsStore.selectedComponentIds;
  selectedComponentIds.push(id);
  componentsStore.setSelectedComponentId(_.uniq(selectedComponentIds));
}

function onOutlinePanToComponent(id: string) {
  componentsStore.panTargetComponentId = id;
}

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  // TODO: make actually do something, probably also want to handle different types
  contextMenuRef.value?.open(rightClickEventInfo.e, true);
}

function triggerDeleteSelection() {
  // TODO: decide if modal is necessary
  confirmDeleteModalRef.value?.open();
}
function onConfirmDelete() {
  // TODO: show loading in modal, and close after complete
  executeDeleteSelection();
  confirmDeleteModalRef.value?.close();
}
async function executeDeleteSelection() {
  if (selectedEdgeId.value) {
    await componentsStore.DELETE_EDGE(selectedEdgeId.value);
  } else if (selectedComponentIds.value) {
    for (const componentId of selectedComponentIds.value) {
      await componentsStore.DELETE_COMPONENT(componentId);
    }
  }
}

watch(
  () => [selectedComponentIds.value, selectedEdgeId.value],
  () => {
    if (selectedComponentIds.value.length > 0) {
      const selectedComponentsKeys = _.map(selectedComponentIds.value, (c) => {
        const component = componentsStore.componentsById[c];

        return component.isGroup
          ? DiagramGroupData.generateUniqueKey(component.nodeId)
          : DiagramNodeData.generateUniqueKey(component.nodeId);
      });

      diagramRef.value?.setSelection(selectedComponentsKeys);
    } else if (selectedEdgeId.value) {
      diagramRef.value?.setSelection(
        DiagramEdgeData.generateUniqueKey(selectedEdgeId.value),
      );
    } else {
      diagramRef.value?.clearSelection();
    }
  },
);

function onGroupElements({ group, elements }: GroupEvent) {
  if (group.def.nodeType === "aggregationFrame") {
    const groupSchemaId =
      componentsStore.componentsByNodeId[group.def.id].schemaVariantId;
    elements = _.filter(elements, (e) => {
      const elementSchemaId =
        componentsStore.componentsByNodeId[e.def.id].schemaVariantId;

      return elementSchemaId === groupSchemaId;
    });
  }

  for (const element of elements) {
    componentsStore.CONNECT_COMPONENT_TO_FRAME(element.def.id, group.def.id);
  }
}
</script>
