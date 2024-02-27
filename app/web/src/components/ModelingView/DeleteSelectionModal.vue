<template>
  <Modal
    ref="modalRef"
    :title="deletionBlockedReason ? 'Cannot delete selection' : 'Are you sure?'"
  >
    <div class="max-h-[80vh] overflow-hidden flex flex-col">
      <template v-if="deletionBlockedReason">
        <Stack spacing="sm">
          <ErrorMessage :message="deletionBlockedReason" />

          <VButton icon="x" tone="shade" variant="ghost" @click="close">
            Cancel
          </VButton>
        </Stack>
      </template>
      <template v-else>
        <template v-if="selectedEdge">
          <p>You're about to delete the following edge:</p>
          <EdgeCard :edgeId="selectedEdge.id" />
        </template>
        <template v-else>
          <div class="pb-xs">
            You are about to delete
            {{
              componentsStore.deletableSelectedComponents.length > 1
                ? "the following components"
                : "this component"
            }}:
          </div>
          <div
            :class="
              clsx(
                'flex-grow overflow-y-auto border-neutral-300 dark:border-neutral-700 p-xs',
                componentsStore.deletableSelectedComponents.length > 1 &&
                  'border',
              )
            "
          >
            <Stack spacing="xs">
              <ComponentCard
                v-for="component in componentsStore.deletableSelectedComponents"
                :key="component.id"
                :componentId="component.id"
              />
            </Stack>
          </div>
        </template>
        <div class="py-xs">
          Items that exist on HEAD will be marked for deletion, and removed from
          the model when this change set is merged. Items that were created in
          this change set will be deleted immediately.
        </div>

        <div class="flex gap-sm">
          <VButton icon="x" tone="shade" variant="ghost" @click="close">
            Cancel
          </VButton>
          <VButton
            icon="trash"
            tone="destructive"
            class="flex-grow"
            @click="onConfirmDelete"
          >
            Confirm
          </VButton>
        </div>
      </template>
    </div>
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

import clsx from "clsx";
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
  close();
  if (componentsStore.selectedEdgeId) {
    await componentsStore.DELETE_EDGE(componentsStore.selectedEdgeId);
  } else if (componentsStore.selectedComponentIds) {
    await componentsStore.DELETE_COMPONENTS(
      componentsStore.selectedComponentIds,
    );
  }
  componentsStore.setSelectedComponentId(null);
}

const modelingEventBus = componentsStore.eventBus;
onMounted(() => {
  modelingEventBus.on("deleteSelection", open);
  window.addEventListener("keydown", onKeyDown);
});
onBeforeUnmount(() => {
  modelingEventBus.off("deleteSelection", open);
  window.removeEventListener("keydown", onKeyDown);
});

const onKeyDown = async (e: KeyboardEvent) => {
  if (
    e.key === "Enter" &&
    !deletionBlockedReason.value &&
    modalRef.value?.isOpen
  ) {
    onConfirmDelete();
  }
};

defineExpose({ open, close });
</script>
