<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <!-- left panel - outline + asset palette -->
  <ResizablePanel
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
      />
    </template>
  </ResizablePanel>

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
  <ResizablePanel
    rememberSizeKey="details-panel"
    side="right"
    :defaultSize="430"
    :minSize="430"
  >
    <div class="h-full overflow-hidden relative">
      <EdgeDetailsPanel v-if="selectedEdge" />
      <ComponentDetails
        v-else-if="selectedComponent"
        :key="selectedComponent.id"
      />
      <MultiSelectDetailsPanel v-else-if="selectedComponentIds.length > 1" />
      <NoSelectionDetailsPanel v-else />
    </div>
  </ResizablePanel>

  <ModelingRightClickMenu ref="contextMenuRef" />
  <DeleteSelectionModal />
  <RestoreSelectionModal />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, ref } from "vue";
import { ResizablePanel } from "@si/vue-lib/design-system";
import ComponentDetails from "@/components/ComponentDetails.vue";
import { useComponentsStore } from "@/store/components.store";
import { useFixesStore } from "@/store/fixes.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import FixProgressOverlay from "@/components/FixProgressOverlay.vue";
import { usePresenceStore } from "@/store/presence.store";
import { useSecretsStore } from "@/store/secrets.store";
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

const openCollapsible = ref(true);

onMounted(() => {
  if (changeSetStore.headSelected) {
    openCollapsible.value = !!window.localStorage.getItem("applied-changes");
    window.localStorage.removeItem("applied-changes");
  } else {
    openCollapsible.value = false;
  }
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

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  contextMenuRef.value?.open(rightClickEventInfo.e, true);
}

function onOutlineRightClick(e: MouseEvent) {
  contextMenuRef.value?.open(e, true);
}
</script>
