<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <!-- left panel - outline + asset palette -->
  <component
    :is="ResizablePanel"
    ref="leftResizablePanelRef"
    :minSize="250"
    rememberSizeKey="change-set-and-asset"
    side="left"
  >
    <template #subpanel1>
      <DiagramOutline
        :actionsAreRunning="actionsAreRunning"
        @right-click-item="onOutlineRightClick"
      />
    </template>
    <template #subpanel2>
      <AssetPalette class="border-t dark:border-neutral-600" />
    </template>
  </component>

  <div
    v-if="
      featureFlagsStore.REBAC &&
      changeSetsStore.selectedChangeSet?.status ===
        ChangeSetStatus.NeedsApproval
    "
    :class="
      clsx(
        'grow flex flew-row items-center justify-center',
        themeClasses('bg-shade-0', 'bg-neutral-800'),
      )
    "
  >
    <InsetApprovalModal />
  </div>
  <ModelingDiagram
    v-else
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
      <EdgeDetailsPanel
        v-if="selectedEdge"
        :menuSelected="contextMenuRef?.isOpen"
        @openMenu="onThreeDotMenuClick"
      />
      <ComponentDetails
        v-else-if="selectedComponent"
        :key="selectedComponent.def.id"
        :component="selectedComponent"
        :menuSelected="contextMenuRef?.isOpen as boolean"
        @openMenu="onThreeDotMenuClick"
      />
      <MultiSelectDetailsPanel
        v-else-if="selectedComponentIds.length > 1"
        :menuSelected="contextMenuRef?.isOpen"
        @openMenu="onThreeDotMenuClick"
      />
      <NoSelectionDetailsPanel v-else />
    </div>
  </component>

  <ModelingRightClickMenu ref="contextMenuRef" />
  <DeleteSelectionModal />
  <RestoreSelectionModal />
  <EraseSelectionModal />
  <CommandModal />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { ResizablePanel, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import ComponentDetails from "@/components/ComponentDetails.vue";
import { useComponentsStore } from "@/store/components.store";
import { useActionsStore } from "@/store/actions.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { usePresenceStore } from "@/store/presence.store";
// import ActionProgressOverlay from "@/components/ActionProgressOverlay.vue";
import { useSecretsStore } from "@/store/secrets.store";
import EraseSelectionModal from "@/components/ModelingView/EraseSelectionModal.vue";
import { useStatusStore } from "@/store/status.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import ModelingDiagram from "../ModelingDiagram/ModelingDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import {
  DiagramGroupData,
  DiagramNodeData,
  RightClickElementEvent,
} from "../ModelingDiagram/diagram_types";
import DiagramOutline from "../DiagramOutline/DiagramOutline.vue";
import EdgeDetailsPanel from "../EdgeDetailsPanel.vue";
import MultiSelectDetailsPanel from "../MultiSelectDetailsPanel.vue";
import NoSelectionDetailsPanel from "../NoSelectionDetailsPanel.vue";
import ModelingRightClickMenu from "../ModelingView/ModelingRightClickMenu.vue";
import DeleteSelectionModal from "../ModelingView/DeleteSelectionModal.vue";
import RestoreSelectionModal from "../ModelingView/RestoreSelectionModal.vue";
import CommandModal from "./CommandModal.vue";
import InsetApprovalModal from "../InsetApprovalModal.vue";

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const actionsStore = useActionsStore();
const presenceStore = usePresenceStore();
const _secretsStore = useSecretsStore(); // adding this so we fetch once
const statusStore = useStatusStore();
const featureFlagsStore = useFeatureFlagsStore();

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
  statusStore.FETCH_DVU_ROOTS();
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
  component: DiagramGroupData | DiagramNodeData;
}) {
  contextMenuRef.value?.open(ev.mouse, true, ev.component.def.position);
}

function onThreeDotMenuClick(mouse: MouseEvent) {
  contextMenuRef.value?.open(mouse, false);
}
</script>
