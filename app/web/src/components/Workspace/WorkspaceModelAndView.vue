<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <!-- Left Panel - views drawer and outline + asset palette -->
  <section
    class="absolute flex flex-row h-full"
    :style="{
      left: drawerLeftPos + 'px',
      transition: 'left 0.15s ease-out',
    }"
  >
    <LeftPanelDrawer
      :changeSetId="changeSetsStore.selectedChangeSetId"
      @closed="toggleDrawer"
    />
    <component
      :is="ResizablePanel"
      ref="leftResizablePanelRef"
      :defaultSize="320"
      :minSize="250"
      rememberSizeKey="change-set-and-asset"
      side="left"
      @sizeSet="leftPanelSize"
    >
      <template #subpanel1>
        <DiagramOutline
          :actionsAreRunning="actionsAreRunning"
          :toggleDrawer="toggleDrawer"
          :leftDrawerOpen="presenceStore.leftDrawerOpen"
          @right-click-item="onOutlineRightClick"
        />
      </template>
      <template #subpanel2>
        <AssetPalette class="border-t dark:border-neutral-600" />
      </template>
    </component>
  </section>

  <!-- Middle Area - ModelingDiagram or InsetApprovalModal -->
  <div
    v-if="
      featureFlagsStore.REBAC &&
      changeSetsStore.selectedChangeSet?.status !== ChangeSetStatus.Open
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
    @close-right-click-menu="closeRightClickMenu"
  />

  <!-- Right Panel - selection details -->
  <section class="absolute right-0 h-full">
    <component
      :is="ResizablePanel"
      ref="rightResizablePanelRef"
      class="h-full"
      :defaultSize="320"
      :minSize="320"
      rememberSizeKey="details-panel"
      side="right"
      @sizeSet="rightPanelSize"
    >
      <div class="h-full overflow-hidden relative">
        <EdgeDetailsPanel
          v-if="selectedEdge"
          :menuSelected="contextMenuRef?.isOpen ?? false"
          @openMenu="onThreeDotMenuClick"
        />
        <ComponentDetails
          v-else-if="selectedComponent"
          :key="selectedComponent.def.id"
          :component="selectedComponent"
          :menuSelected="contextMenuRef?.isOpen as boolean ?? false"
          @openMenu="onThreeDotMenuClick"
        />
        <MultiSelectDetailsPanel
          v-else-if="selectedComponentIds.length > 1"
          :menuSelected="contextMenuRef?.isOpen ?? false"
          @openMenu="onThreeDotMenuClick"
        />
        <NoSelectionDetailsPanel v-else />
      </div>
    </component>
  </section>

  <!-- Modals and Menus outside of the flow of the page -->
  <ModelingRightClickMenu ref="contextMenuRef" />
  <DeleteSelectionModal />
  <RestoreSelectionModal />
  <EraseSelectionModal />
  <TemplateSelectionModal
    v-if="featureFlagsStore.TEMPLATE_MGMT_FUNC_GENERATION"
  />
  <CommandModal />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onBeforeMount, onBeforeUnmount, onMounted, ref } from "vue";
import { ResizablePanel, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { IRect } from "konva/lib/types";
import ComponentDetails from "@/components/ComponentDetails.vue";
import { useActionsStore } from "@/store/actions.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { usePresenceStore } from "@/store/presence.store";
// import ActionProgressOverlay from "@/components/ActionProgressOverlay.vue";
import { useSecretsStore } from "@/store/secrets.store";
import EraseSelectionModal from "@/components/ModelingView/EraseSelectionModal.vue";
import { useStatusStore } from "@/store/status.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useViewsStore } from "@/store/views.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useRouterStore } from "@/store/router.store";
import { useAssetStore } from "@/store/asset.store";
import { useFuncStore } from "@/store/func/funcs.store";
import LeftPanelDrawer from "../LeftPanelDrawer.vue";
import ModelingDiagram from "../ModelingDiagram/ModelingDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import {
  DiagramElementData,
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
import TemplateSelectionModal from "../ModelingView/TemplateSelectionModal.vue";
import CommandModal from "./CommandModal.vue";
import InsetApprovalModal from "../InsetApprovalModal.vue";

const changeSetsStore = useChangeSetsStore();
const viewStore = useViewsStore();
const actionsStore = useActionsStore();
const presenceStore = usePresenceStore();
const assetStore = useAssetStore();
const _secretsStore = useSecretsStore(); // adding this so we fetch once
const statusStore = useStatusStore();
const funcStore = useFuncStore();
const featureFlagsStore = useFeatureFlagsStore();

const actionsAreRunning = computed(
  () =>
    actionsStore.actionsAreInProgress ||
    changeSetsStore.getRequestStatus("APPLY_CHANGE_SET").value.isPending,
);

const leftResizablePanelRef = ref();
const rightResizablePanelRef = ref();

const leftPanelSize = (size: number) => {
  presenceStore.leftResizePanelWidth = size;
};

const rightPanelSize = (size: number) => {
  presenceStore.rightResizePanelWidth = size;
};

const drawerLeftPos = computed(() => {
  if (presenceStore.leftDrawerOpen) return 0;
  else return -230;
});

const toggleDrawer = () => {
  presenceStore.leftDrawerOpen = !presenceStore.leftDrawerOpen;
};

const onKeyDown = async (e: KeyboardEvent) => {
  if (presenceStore.leftDrawerOpen && e.key === "Escape") {
    toggleDrawer();
  }

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

const routeStore = useRouterStore();

onBeforeMount(async () => {
  // get to first paint ASAP
  await Promise.all([
    viewStore.LIST_VIEWS(),
    assetStore.LOAD_SCHEMA_VARIANT_LIST(),
  ]);
  let viewId;
  if (routeStore.currentRoute?.params.viewId)
    viewId = routeStore.currentRoute?.params.viewId as string;

  await Promise.all([
    viewStore.FETCH_VIEW(viewId), // draws the minimal diagram, later gets all geometry and component
    funcStore.FETCH_FUNC_LIST(), // required for actions of a selected component to work
  ]);

  const key = `${changeSetsStore.selectedChangeSetId}_selected_component`;
  const lastId = window.localStorage.getItem(key);
  window.localStorage.removeItem(key);
  if (
    lastId &&
    Object.values(viewStore.selectedComponentIds).filter(Boolean).length === 0
  ) {
    viewStore.setSelectedComponentId(lastId);
  }

  // filling out the rest of the needed data
  await Promise.all([
    actionsStore.LOAD_ACTIONS(),
    actionsStore.LOAD_ACTION_HISTORY(),
    statusStore.FETCH_DVU_ROOTS(),
  ]);
});

onMounted(async () => {
  window.addEventListener("keydown", onKeyDown);
});

onBeforeUnmount(() => {
  window.removeEventListener("keydown", onKeyDown);
});

const contextMenuRef = ref<InstanceType<typeof ModelingRightClickMenu>>();

const selectedComponentIds = computed(() => viewStore.selectedComponentIds);

const selectedEdge = computed(() => viewStore.selectedEdge);
const selectedComponent = computed<
  DiagramGroupData | DiagramNodeData | undefined
>(() =>
  viewStore.selectedComponent?.def.componentType !== ComponentType.View
    ? (viewStore.selectedComponent as DiagramGroupData | DiagramNodeData)
    : undefined,
);

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  const id = rightClickEventInfo.element.def.id;
  let component: DiagramGroupData | DiagramNodeData | undefined;
  let position: IRect | undefined;

  if ("componentType" in rightClickEventInfo.element.def)
    component = rightClickEventInfo.element as
      | DiagramGroupData
      | DiagramNodeData;

  if (component) {
    position = structuredClone(
      component.def.isGroup ? viewStore.components[id] : viewStore.groups[id],
    );
  }
  if (position) position.y += position.height / 2;
  contextMenuRef.value?.open(rightClickEventInfo.e, true, position);
}

function onOutlineRightClick(ev: {
  mouse: MouseEvent;
  component: DiagramElementData;
}) {
  const id = ev.component.def.id;
  let component: DiagramGroupData | DiagramNodeData | undefined;
  if ("componentType" in ev.component.def)
    component = ev.component as DiagramGroupData | DiagramNodeData;

  let position: IRect | undefined;
  if (component) {
    position = component.def.isGroup
      ? viewStore.components[id]
      : viewStore.groups[id];
  }
  contextMenuRef.value?.open(ev.mouse, true, position);
}

function onThreeDotMenuClick(mouse: MouseEvent) {
  contextMenuRef.value?.open(mouse, false);
}

function closeRightClickMenu() {
  contextMenuRef.value?.close();
}
</script>
