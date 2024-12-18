<template>
  <Modal ref="modalRef" :title="'Are you sure?'">
    <div class="max-h-[70vh] overflow-hidden flex flex-col">
      <template v-if="selectedEdge">
        <p>You're about to delete the following edge:</p>
        <EdgeCard :edgeId="selectedEdge.id" />
      </template>
      <template v-else-if="allErasableViews">
        <div class="pb-xs">
          You are about to remove
          {{ erasableViews.length > 1 ? "the following views" : "this view" }}
          from {{ viewStore.selectedView?.name }}
        </div>
        <div
          :class="
            clsx(
              'flex-grow overflow-y-auto border-neutral-300 dark:border-neutral-700 p-xs',
              erasableViews.length > 1 && 'border',
            )
          "
        >
          <Stack spacing="xs">
            <ComponentCard
              v-for="component in erasableViews"
              :key="component.def.id"
              :component="component"
            />
          </Stack>
        </div>
      </template>
      <template v-else>
        <div class="pb-xs">
          You are about to delete
          {{
            deletableComponentsInView.length > 1
              ? "the following components"
              : "this component"
          }}:
        </div>
        <div
          :class="
            clsx(
              'flex-grow overflow-y-auto border-neutral-300 dark:border-neutral-700 p-xs',
              deletableComponentsInView.length > 1 && 'border',
            )
          "
        >
          <Stack spacing="xs">
            <ComponentCard
              v-for="component in deletableComponentsInView"
              :key="component.def.id"
              :component="component"
            />
          </Stack>
        </div>
      </template>
      <div class="px-2xs py-xs">
        <template v-if="selectedEdge">
          <p class="text-xs mt-sm">
            Items that exist on HEAD will be marked for deletion, and removed
            from the model when this change set is merged. Items that were
            created in this change set will be deleted immediately.
          </p>
        </template>
        <template v-else-if="!allErasableViews">
          <VormInput v-model="removeOrDelete" noLabel type="radio">
            <VormInputOption :value="DELETE">
              <p class="text-xs mt-sm">
                <strong>Delete</strong>: Items that exist on HEAD will be marked
                for deletion, and removed from the model when this change set is
                merged. Items that were created in this change set will be
                deleted immediately.
              </p>
            </VormInputOption>
            <VormInputOption :value="REMOVE">
              <p class="text-xs my-sm">
                <strong>Remove</strong>: Items will be removed from this view of
                your diagram. It will remain in other views.
              </p>
            </VormInputOption>
          </VormInput>
        </template>
      </div>

      <div class="flex gap-sm">
        <VButton tone="shade" variant="ghost" @click="close"> Cancel </VButton>
        <VButton
          icon="trash"
          tone="destructive"
          class="flex-grow"
          @click="onConfirmDelete"
        >
          <template v-if="selectedEdge"> Delete </template>
          <template v-else> Confirm </template>
        </VButton>
      </div>
    </div>
  </Modal>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  Modal,
  Stack,
  useModal,
  VButton,
  VormInput,
  VormInputOption,
} from "@si/vue-lib/design-system";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";

import clsx from "clsx";
import { useComponentsStore } from "@/store/components.store";
import { useViewsStore } from "@/store/views.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import ComponentCard from "../ComponentCard.vue";
import EdgeCard from "../EdgeCard.vue";

const componentsStore = useComponentsStore();
const viewStore = useViewsStore();
const selectedEdge = computed(() => viewStore.selectedEdge);

const modalRef = ref<InstanceType<typeof Modal>>();
const { open: openModal, close } = useModal(modalRef);

const deletableComponentsInView = computed(() => {
  return viewStore.deletableSelectedComponents.filter((c) => {
    if (viewStore.components[c.def.id]) return true;
    if (viewStore.groups[c.def.id]) return true;
    return false;
  });
});

const erasableViews = computed(() => {
  const selectedViews = viewStore.selectedComponents.filter(
    (c) => c.def.componentType === ComponentType.View,
  );
  const view = Object.keys(viewStore.viewNodes);
  return selectedViews.filter((v) => view.includes(v.def.id));
});

const allErasableViews = computed(() => {
  const allViews = viewStore.selectedComponents.every(
    (c) => c.def.componentType === ComponentType.View,
  );
  return (
    allViews &&
    erasableViews.value.length === viewStore.selectedComponents.length &&
    viewStore.selectedComponents.length > 0
  );
});

function open() {
  // event is triggered regardless of selection
  // in some cases we may want to ignore it
  if (selectedEdge.value) {
    if (selectedEdge.value?.changeStatus === "deleted") return;
  } else if (allErasableViews.value) {
    // we can erase all these views
  } else {
    // TODO: more logic to decide if modal is necessary for other situations
    if (!deletableComponentsInView.value.length) return;
  }

  openModal();
}
const DELETE = "delete";
const REMOVE = "remove";
const removeOrDelete = ref(DELETE);

async function onConfirmDelete() {
  close();
  if (selectedEdge.value || removeOrDelete.value === DELETE) {
    if (
      viewStore.selectedEdgeId &&
      viewStore.selectedEdge?.toSocketId &&
      viewStore.selectedEdge?.fromSocketId
    ) {
      const resp = await componentsStore.DELETE_EDGE(
        viewStore.selectedEdgeId,
        viewStore.selectedEdge?.toSocketId,
        viewStore.selectedEdge?.fromSocketId,
        viewStore.selectedEdge?.toComponentId,
        viewStore.selectedEdge?.fromComponentId,
      );
      if (resp.result.success) {
        viewStore.selectedEdgeId = null;
      }
    } else if (viewStore.selectedViewId && erasableViews.value.length > 0) {
      await viewStore.REMOVE_VIEW_FROM(
        viewStore.selectedViewId,
        erasableViews.value.map((v) => v.def.id),
      );
    } else if (deletableComponentsInView.value.length > 0) {
      await componentsStore.DELETE_COMPONENTS([
        ...new Set(deletableComponentsInView.value.map((c) => c.def.id)),
      ]);
    }
  } else if (removeOrDelete.value === REMOVE) {
    if (
      viewStore.selectedViewId &&
      deletableComponentsInView.value.length > 0
    ) {
      await viewStore.REMOVE_FROM(viewStore.selectedViewId, [
        ...new Set(deletableComponentsInView.value.map((c) => c.def.id)),
      ]);
    }
  }
  viewStore.setSelectedComponentId(null);
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
