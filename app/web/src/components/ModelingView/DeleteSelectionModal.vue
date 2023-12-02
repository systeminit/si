<template>
  <Modal ref="modalRef" :title="deletionBlockedReason || 'Are you sure?'">
    <template v-if="deletionBlockedReason">
      <Stack spacing="sm">
        <ErrorMessage :message="deletionBlockedReason" />

        <VButton icon="x" tone="shade" variant="ghost" @click="close">
          Cancel
        </VButton>
      </Stack>
    </template>

    <Stack spacing="sm">
      <template v-if="selectedEdge">
        <p>You're about to delete the following edge:</p>
        <EdgeCard :edgeId="selectedEdge.id" />
      </template>
      <template v-else>
        <p>You're about to delete the following component(s):</p>
        <Stack spacing="xs">
          <ComponentCard
            v-for="component in componentsStore.deletableSelectedComponents"
            :key="component.id"
            :componentId="component.id"
          />
        </Stack>
      </template>
      <p>
        Items that exist on HEAD will be marked for deletion, and removed from
        the model when this change set is merged. Items that were created in
        this change set will be deleted immediately.
      </p>

      <div class="flex space-x-sm justify-end">
        <VButton icon="x" tone="shade" variant="ghost" @click="close">
          Cancel
        </VButton>
        <VButton icon="trash" tone="destructive" @click="onConfirmDelete">
          Confirm
        </VButton>
      </div>
    </Stack>
  </Modal>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  ErrorMessage,
  Modal,
  Stack,
  VButton,
  useModal,
} from "@si/vue-lib/design-system";
import { computed, onBeforeMount, onBeforeUnmount, onMounted, ref } from "vue";

import { useComponentsStore } from "@/store/components.store";
import ComponentCard from "../ComponentCard.vue";
import EdgeCard from "../EdgeCard.vue";

const componentsStore = useComponentsStore();
const selectedEdge = computed(() => componentsStore.selectedEdge);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const deletionBlockedReason = computed(() => {
  const deletionSubjectHasChildren =
    componentsStore.selectedComponents?.filter((el) => {
      const activeChildren = el.childNodeIds.filter((childId) => {
        const child = componentsStore.componentsByNodeId[childId];
        return _.isNil(child?.deletedInfo);
      });
      return activeChildren.length > 0;
    }).length > 0;
  if (deletionSubjectHasChildren) {
    return "You cannot delete a frame that still has children. Delete them before proceeding.";
  }
  return undefined;
});

function open() {
  // event is triggered regardless of selection
  // in some cases we may want to ignore it
  if (selectedEdge.value) {
    if (selectedEdge.value?.changeStatus === "deleted") return;
  } else {
    // TODO: more logic to decide if modal is necessary for other situations
    if (!componentsStore.deletableSelectedComponents.length) return;
  }

  openModal();
}

async function onConfirmDelete() {
  if (componentsStore.selectedEdgeId) {
    await componentsStore.DELETE_EDGE(componentsStore.selectedEdgeId);
  } else if (componentsStore.selectedComponentIds) {
    await componentsStore.DELETE_COMPONENTS(
      componentsStore.selectedComponentIds,
    );
  }
  componentsStore.setSelectedComponentId(null);
  close();
}

const modelingEventBus = componentsStore.eventBus;
onMounted(() => {
  modelingEventBus.on("deleteSelection", open);
});
onBeforeUnmount(() => {
  modelingEventBus.off("deleteSelection", open);
});

defineExpose({ open, close });
</script>
