<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <!-- left panel - outline + asset palette -->
  <component
    :is="ResizablePanel"
    ref="leftResizablePanelRef"
    :minSize="250"
    rememberSizeKey="changeset-and-asset"
    side="left"
  >
    <template #subpanel1>
      <DiagramOutline
        :actionsAreRunning="actionsAreRunning"
        class=""
        @right-click-item="onOutlineRightClick"
      />
    </template>
    <template #subpanel2>
      <AssetPalette class="border-t dark:border-neutral-600" />
    </template>
  </component>

  <ModelingDiagram
    ref="diagramRef"
    @mouseout="presenceStore.clearCursor"
    @right-click-element="onRightClickElement"
  />

  <!-- Right panel (selection details) -->
  <component
    :is="ResizablePanel"
    ref="rightResizablePanelRef"
    :defaultSize="400"
    :minSize="400"
    rememberSizeKey="details-panel"
    side="right"
  >
    <div class="h-full overflow-hidden relative">
      <EdgeDetailsPanel v-if="selectedEdge" @openMenu="onThreeDotMenuClick" />
      <ComponentDetails
        v-else-if="selectedComponent"
        :key="selectedComponent.id"
        @openMenu="onThreeDotMenuClick"
      />
      <MultiSelectDetailsPanel
        v-else-if="selectedComponentIds.length > 1"
        @openMenu="onThreeDotMenuClick"
      />
      <NoSelectionDetailsPanel v-else />
    </div>
  </component>

  <ModelingRightClickMenu ref="contextMenuRef" />
  <DeleteSelectionModal />
  <RestoreSelectionModal />
  <EraseSelectionModal />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { ResizablePanel } from "@si/vue-lib/design-system";
import ComponentDetails from "@/components/ComponentDetails.vue";
import { useComponentsStore, FullComponent } from "@/store/components.store";
import { useActionsStore } from "@/store/actions.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { usePresenceStore } from "@/store/presence.store";
// import ActionProgressOverlay from "@/components/ActionProgressOverlay.vue";
import { useSecretsStore } from "@/store/secrets.store";
import EraseSelectionModal from "@/components/ModelingView/EraseSelectionModal.vue";
import ModelingDiagram from "../ModelingDiagram/ModelingDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import { RightClickElementEvent } from "../ModelingDiagram/diagram_types";
import DiagramOutline from "../DiagramOutline/DiagramOutline.vue";
import EdgeDetailsPanel from "../EdgeDetailsPanel.vue";
import MultiSelectDetailsPanel from "../MultiSelectDetailsPanel.vue";
import NoSelectionDetailsPanel from "../NoSelectionDetailsPanel.vue";
import ModelingRightClickMenu from "../ModelingView/ModelingRightClickMenu.vue";
import DeleteSelectionModal from "../ModelingView/DeleteSelectionModal.vue";
import RestoreSelectionModal from "../ModelingView/RestoreSelectionModal.vue";

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const actionsStore = useActionsStore();
const presenceStore = usePresenceStore();
const _secretsStore = useSecretsStore(); // adding this so we fetch once

const actionsAreRunning = computed(
  () =>
    actionsStore.actionsAreInProgress ||
    changeSetsStore.getRequestStatus("APPLY_CHANGE_SET").value.isPending,
);

const leftResizablePanelRef = ref();
const rightResizablePanelRef = ref();

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.altKey &&
    e.shiftKey &&
    leftResizablePanelRef.value &&
    rightResizablePanelRef.value
  ) {
    if (
      leftResizablePanelRef.value.collapsed &&
      rightResizablePanelRef.value.collapsed
    ) {
      // Open all panels
      leftResizablePanelRef.value.collapseSet(false);
      rightResizablePanelRef.value.collapseSet(false);
      leftResizablePanelRef.value.subpanelCollapseSet(false);
    } else {
      // Close all panels
      leftResizablePanelRef.value.collapseSet(true);
      rightResizablePanelRef.value.collapseSet(true);
    }
  }
};

onMounted(() => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const contextMenuRef = ref<InstanceType<typeof ModelingRightClickMenu>>();

const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);

const selectedEdge = computed(() => componentsStore.selectedEdge);
const selectedComponent = computed(() => componentsStore.selectedComponent);

// TODO: deal with this...
// watch([diagramNodes, diagramEdges], () => {
//   // TODO: this should be firing off the callback only when we find the matching new node, but we dont have the new ID yet
//   _.each(insertCallbacks, (insertCallback, newNodeId) => {
//     insertCallback();
//     delete insertCallbacks[newNodeId];
//   });
// });

// Nodes that are not resizable have dynamic height based on its rendering objects, we cannot infer that here and honestly it's not a big deal
// So let's hardcode something reasonable that doesn't make the user too much confused when they paste a copy
const NODE_HEIGHT = 200;

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  let position;
  if ("position" in rightClickEventInfo.element.def) {
    position = _.cloneDeep(rightClickEventInfo.element.def.position);
    position.y +=
      (rightClickEventInfo.element.def.size?.height ?? NODE_HEIGHT) / 2;
  }
  contextMenuRef.value?.open(rightClickEventInfo.e, true, position);
}

function onOutlineRightClick(ev: {
  mouse: MouseEvent;
  component: FullComponent;
}) {
  contextMenuRef.value?.open(ev.mouse, true, ev.component.position);
}

function onThreeDotMenuClick(mouse: MouseEvent) {
  contextMenuRef.value?.open(mouse, false);
}
</script>
