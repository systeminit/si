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
      :cursors="presenceStore.diagramCursors"
      @update:pointer="updatePointer"
      @right-click-element="onRightClickElement"
    />
  </div>

  <!-- Right panel (selection details) -->
  <ResizablePanel
    rememberSizeKey="details-panel"
    side="right"
    :defaultSize="380"
    :minSize="350"
    :disableSubpanelResizing="!changesPanelRef?.isOpen"
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

  <Modal ref="actionBlockedModalRef" :title="actionBlockedModalTitle">
    <Stack spacing="sm">
      <p>
        {{ actionBlockedModalText }}
      </p>

      <div class="flex space-x-sm justify-end">
        <VButton tone="action" @click="closeDeleteBlockedModal"> Ok</VButton>
      </div>
    </Stack>
  </Modal>

  <ModelingRightClickMenu ref="contextMenuRef" />
  <DeleteSelectionModal />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, ref, watch } from "vue";
import {
  Collapsible,
  VButton,
  Modal,
  Stack,
  ResizablePanel,
} from "@si/vue-lib/design-system";
import ComponentDetails from "@/components/ComponentDetails.vue";
import { useComponentsStore } from "@/store/components.store";
import { useFixesStore } from "@/store/fixes.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import FixProgressOverlay from "@/components/FixProgressOverlay.vue";
import { usePresenceStore } from "@/store/presence.store";
import ModelingDiagram from "../ModelingDiagram/ModelingDiagram.vue";
import AssetPalette from "../AssetPalette.vue";
import {
  RightClickElementEvent,
  MovePointerEvent,
} from "../ModelingDiagram/diagram_types";
import ComponentOutline from "../ComponentOutline/ComponentOutline.vue";
import EdgeDetailsPanel from "../EdgeDetailsPanel.vue";
import MultiSelectDetailsPanel from "../MultiSelectDetailsPanel.vue";
import NoSelectionDetailsPanel from "../NoSelectionDetailsPanel.vue";
import ModelingRightClickMenu from "../ModelingView/ModelingRightClickMenu.vue";
import DeleteSelectionModal from "../ModelingView/DeleteSelectionModal.vue";

const changeSetStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const fixesStore = useFixesStore();
const presenceStore = usePresenceStore();

const modelingEventBus = componentsStore.eventBus;

const updatePointer = (pos: MovePointerEvent) => {
  presenceStore.updateCursor(pos);
};

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

const selectedEdgeId = computed(() => componentsStore.selectedEdgeId);
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

const actionBlockedModalRef = ref<InstanceType<typeof Modal>>();
const actionBlockedModalTitle = ref<string>();
const actionBlockedModalText = ref<string>();

function closeDeleteBlockedModal() {
  actionBlockedModalRef.value?.close();
}

async function triggerRestoreSelection() {
  if (selectedEdgeId.value) {
    await componentsStore.RESTORE_EDGE(selectedEdgeId.value);
  } else if (selectedComponentIds.value) {
    // Block restoring child of deleted frame
    const parentIds = _.compact(
      _.map(
        selectedComponentIds.value,
        (id) => componentsStore.componentsById[id]?.parentId,
      ),
    );

    const hasDeletedParent = parentIds.find(
      (id) => !_.isNil(componentsStore.componentsById[id]?.deletedInfo),
    );

    if (hasDeletedParent) {
      actionBlockedModalTitle.value = "Can't restore component";
      actionBlockedModalText.value =
        "You cannot restore a component inside a deleted frame. Restore the parent before restoring its children.";

      actionBlockedModalRef.value?.open();
      return;
    }

    await componentsStore.RESTORE_COMPONENTS(selectedComponentIds.value);
  }
}

function onRightClickElement(rightClickEventInfo: RightClickElementEvent) {
  contextMenuRef.value?.open(rightClickEventInfo.e, true);
}

function onOutlineRightClick(e: MouseEvent) {
  contextMenuRef.value?.open(e, true);
}

const changesPanelRef = ref<InstanceType<typeof Collapsible>>();
</script>
