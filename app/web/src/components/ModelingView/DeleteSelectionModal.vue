<template>
  <Modal ref="modalRef" :title="'Are you sure?'">
    <div class="max-h-[80vh] overflow-hidden flex flex-col">
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
    </div>
  </Modal>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { Modal, Stack, useModal, VButton } from "@si/vue-lib/design-system";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import clsx from "clsx";
import { useComponentsStore } from "@/store/components.store";
import ComponentCard from "../ComponentCard.vue";
import EdgeCard from "../EdgeCard.vue";

const componentsStore = useComponentsStore();
const selectedEdge = computed(() => componentsStore.selectedEdge);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

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
  if (
    componentsStore.selectedEdgeId &&
    componentsStore.selectedEdge?.toSocketId &&
    componentsStore.selectedEdge?.fromSocketId
  ) {
    await componentsStore.DELETE_EDGE(
      componentsStore.selectedEdgeId,
      componentsStore.selectedEdge?.toSocketId,
      componentsStore.selectedEdge?.fromSocketId,
      componentsStore.selectedEdge?.toComponentId,
      componentsStore.selectedEdge?.fromComponentId,
    );
  } else if (componentsStore.deletableSelectedComponents.length > 0) {
    await componentsStore.DELETE_COMPONENTS(
      componentsStore.deletableSelectedComponents.map((c) => c.id),
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
  if (e.key === "Enter" && modalRef.value?.isOpen) {
    onConfirmDelete();
  }
};

defineExpose({ open, close });
</script>
