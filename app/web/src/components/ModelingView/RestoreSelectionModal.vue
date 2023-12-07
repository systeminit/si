/* This is set up to mirror the delete selection modal, but it's a bit weird
since currently we don't show a confirmation step to restore. We may want to
move some of the logic to the store and use a more general error popup... but
we'll see. */

<template>
  <Modal
    ref="modalRef"
    :title="restoreBlockedReason ? 'Cannot restore selection' : 'Are you sure?'"
  >
    <template v-if="restoreBlockedReason">
      <Stack spacing="sm">
        <ErrorMessage :message="restoreBlockedReason" />

        <VButton icon="x" tone="shade" variant="ghost" @click="close">
          Cancel
        </VButton>
      </Stack>
    </template>
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
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import { useComponentsStore } from "@/store/components.store";

const componentsStore = useComponentsStore();
const selectedEdge = computed(() => componentsStore.selectedEdge);

const selectedComponentIds = computed(
  () => componentsStore.selectedComponentIds,
);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const restoreBlockedReason = computed(() => {
  if (!selectedComponentIds.value) return undefined;
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
    return "You cannot restore a component inside a deleted frame. Restore the parent before restoring its children.";
  }
  return undefined;
});

function open() {
  // event is triggered regardless of selection
  // in some cases we may want to ignore it

  if (!selectedEdge.value && !selectedComponentIds.value.length) return;

  if (restoreBlockedReason.value) {
    openModal();
  } else {
    onConfirmRestore();
  }
}

async function onConfirmRestore() {
  if (componentsStore.selectedEdgeId) {
    await componentsStore.RESTORE_EDGE(componentsStore.selectedEdgeId);
  } else if (componentsStore.selectedComponentIds) {
    await componentsStore.RESTORE_COMPONENTS(
      componentsStore.selectedComponentIds,
    );
  }
  componentsStore.setSelectedComponentId(null);
  close();
}

const modelingEventBus = componentsStore.eventBus;
onMounted(() => {
  modelingEventBus.on("restoreSelection", open);
});
onBeforeUnmount(() => {
  modelingEventBus.off("restoreSelection", open);
});

defineExpose({ open, close });
</script>
