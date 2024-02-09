<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <!-- left panel - outline + asset palette -->
  <component
    :is="
      featureFlagsStore.RESIZABLE_PANEL_UPGRADE
        ? ResizablePanel
        : ResizablePanelOld
    "
    ref="leftResizablePanelRef"
    rememberSizeKey="changeset-and-asset"
    side="left"
    :minSize="250"
  >
    <template #subpanel1>
      <ComponentOutline
        class=""
        :fixesAreRunning="fixesAreRunning"
        @right-click-item="onOutlineRightClick"
      />
    </template>
    <template #subpanel2>
      <AssetPalette
        class="border-t dark:border-neutral-600"
        :fixesAreRunning="fixesAreRunning"
        :collapsed="leftResizablePanelRef?.subpanelCollapsed || false"
        @collapse-toggle="leftResizablePanelRef?.subpanelCollapseToggle"
      />
    </template>
  </component>

  <div
    class="grow h-full relative bg-neutral-50 dark:bg-neutral-900"
    @mouseout="presenceStore.clearCursor"
  >
    <!--div
      v-if="!statusStore.globalStatus.isUpdating && isViewMode"
      :class="
        clsx(
          'absolute z-20 left-0 right-0 mx-4 mt-3 p-xs',
          'bg-white dark:bg-neutral-800 dark:text-white border border-neutral-300 dark:border-neutral-600',
          'shadow-md rounded-md font-bold text-center',
        )
      "
    >
      <ReadOnlyBanner show-refresh-all-button />
    </div-->
    <FixProgressOverlay />
    <ModelingDiagram
      ref="diagramRef"
      @right-click-element="onRightClickElement"
    />
  </div>

  <!-- Right panel (selection details) -->
  <component
    :is="
      featureFlagsStore.RESIZABLE_PANEL_UPGRADE
        ? ResizablePanel
        : ResizablePanelOld
    "
    ref="rightResizablePanelRef"
    rememberSizeKey="details-panel"
    side="right"
    :defaultSize="400"
    :minSize="400"
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
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { ResizablePanel, ResizablePanelOld } from "@si/vue-lib/design-system";
import ComponentDetails from "@/components/ComponentDetails.vue";
import { useComponentsStore, FullComponent } from "@/store/components.store";
import { useFixesStore } from "@/store/fixes.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import FixProgressOverlay from "@/components/FixProgressOverlay.vue";
import { usePresenceStore } from "@/store/presence.store";
import { useSecretsStore } from "@/store/secrets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ModelingDiagram from "../ModelingDiagram/ModelingDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import { RightClickElementEvent } from "../ModelingDiagram/diagram_types";
import ComponentOutline from "../ComponentOutline/ComponentOutline.vue";
import EdgeDetailsPanel from "../EdgeDetailsPanel.vue";
import MultiSelectDetailsPanel from "../MultiSelectDetailsPanel.vue";
import NoSelectionDetailsPanel from "../NoSelectionDetailsPanel.vue";
import ModelingRightClickMenu from "../ModelingView/ModelingRightClickMenu.vue";
import DeleteSelectionModal from "../ModelingView/DeleteSelectionModal.vue";
import RestoreSelectionModal from "../ModelingView/RestoreSelectionModal.vue";

const featureFlagsStore = useFeatureFlagsStore();
const changeSetStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const fixesStore = useFixesStore();
const presenceStore = usePresenceStore();
const _secretsStore = useSecretsStore(); // adding this so we fetch once

const fixesAreRunning = computed(
  () =>
    fixesStore.fixesAreInProgress ||
    changeSetStore.getRequestStatus("APPLY_CHANGE_SET").value.isPending,
);

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.key === "Alt" &&
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

const openCollapsible = ref(true);
onMounted(() => {
  if (changeSetStore.headSelected) {
    openCollapsible.value = !!window.localStorage.getItem("applied-changes");
    window.localStorage.removeItem("applied-changes");
  } else {
    openCollapsible.value = false;
  }

  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const leftResizablePanelRef = ref();
const rightResizablePanelRef = ref();

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
