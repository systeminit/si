<template>
  <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import plur from "plur";
import { useComponentsStore } from "@/store/components.store";
import { ComponentType } from "@/api/sdf/dal/diagram";
import { useChangeSetsStore } from "@/store/change_sets.store";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();

const {
  selectedComponentId,
  selectedComponentIds,
  selectedComponent,
  selectedComponents,
  deletableSelectedComponents,
  restorableSelectedComponents,
  selectedEdgeId,
  selectedEdge,
} = storeToRefs(componentsStore);

function typeDisplayName(action = "delete") {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.componentType === ComponentType.Component)
      return "Component";
    else return "Frame";
  } else if (selectedComponentIds.value.length) {
    const components =
      action === "delete"
        ? deletableSelectedComponents.value
        : restorableSelectedComponents.value;

    for (const c of components) {
      if (c.componentType === ComponentType.Component) return "Component"; // if we have both frames and components, just use the word component
    }

    return "Frame";
  } else {
    return "Component";
  }
}

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;
  if (selectedEdgeId.value) {
    // single selected edge
    if (selectedEdge.value?.changeStatus === "deleted") {
      items.push({
        label: "Restore edge",
        icon: "trash-restore",
        onSelect: triggerRestoreSelection,
        disabled,
      });
    } else {
      items.push({
        label: "Delete edge",
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
      });
    }
  } else if (selectedComponentId.value && selectedComponent.value) {
    items.push({
      label: `Copy`,
      icon: "clipboard-copy",
      onSelect: triggerCopySelection,
      disabled,
    });

    // single selected component
    if (selectedComponent.value.toDelete) {
      items.push({
        label: `Restore ${typeDisplayName()} "${
          selectedComponent.value.displayName
        }"`,
        icon: "trash-restore",
        onSelect: triggerRestoreSelection,
        disabled,
      });
    } else {
      items.push({
        label: `Delete ${typeDisplayName()} "${
          selectedComponent.value.displayName
        }"`,
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
      });
    }
  } else if (selectedComponentIds.value.length) {
    items.push({
      label: `Copy ${selectedComponentIds.value.length} Components`,
      icon: "clipboard-copy",
      onSelect: triggerCopySelection,
      disabled,
    });

    // Multiple selected components
    if (deletableSelectedComponents.value.length > 0) {
      items.push({
        label: `Delete ${deletableSelectedComponents.value.length} ${plur(
          typeDisplayName("delete"),
          deletableSelectedComponents.value.length,
        )}`,
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
      });
    }
    if (restorableSelectedComponents.value.length > 0) {
      items.push({
        label: `Restore ${restorableSelectedComponents.value.length} ${plur(
          typeDisplayName("restore"),
          restorableSelectedComponents.value.length,
        )}`,
        icon: "trash-restore",
        onSelect: triggerRestoreSelection,
        disabled,
      });
    }
  }

  if (
    selectedComponents.value.length > 0 &&
    _.every(selectedComponents.value, (c) => (c.ancestorIds?.length || 0) > 0)
  ) {
    items.push({
      label: "Detach from parent(s)",
      icon: "frame",
      onSelect: () => {
        _.each(selectedComponentIds.value, (id) => {
          componentsStore.DETACH_COMPONENT([id]);
        });
      },
      disabled,
    });
  }
  if (
    selectedComponent.value?.hasResource &&
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId
  ) {
    items.push({
      label: "Refresh resource",
      icon: "refresh",
      onSelect: () => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        componentsStore.REFRESH_RESOURCE_INFO(selectedComponent.value!.id);
      },
      disabled,
    });
  }
  return items;
});

function triggerCopySelection() {
  componentsStore.copyingFrom = elementPos.value;
  elementPos.value = null;
}

const modelingEventBus = componentsStore.eventBus;

function triggerDeleteSelection() {
  modelingEventBus.emit("deleteSelection");
  elementPos.value = null;
}

function triggerRestoreSelection() {
  modelingEventBus.emit("restoreSelection");
  elementPos.value = null;
}

const elementPos = ref<{ x: number; y: number } | null>(null);

function open(
  e: MouseEvent,
  anchorToMouse: boolean,
  elementPosition?: { x: number; y: number },
) {
  if (elementPosition) elementPos.value = elementPosition;
  contextMenuRef.value?.open(e, anchorToMouse);
}

defineExpose({ open });
</script>
