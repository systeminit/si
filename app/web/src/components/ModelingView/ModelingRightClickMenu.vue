<template>
  <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import plur from "plur";
import { useComponentsStore } from "@/store/components.store";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useChangeSetsStore } from "@/store/change_sets.store";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();

const {
  selectedComponentId,
  selectedComponentIds,
  selectedComponent,
  selectedComponentsAndChildren,
  deletableSelectedComponents,
  restorableSelectedComponents,
  erasableSelectedComponents,
  selectedEdgeId,
  selectedEdge,
} = storeToRefs(componentsStore);

function typeDisplayName(action = "delete") {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.componentType === ComponentType.Component)
      return "Component";
    else return "Frame";
  } else if (selectedComponentIds.value.length) {
    let components;
    switch (action) {
      case "delete":
        components = deletableSelectedComponents.value;
        break;
      case "erase":
        components = erasableSelectedComponents.value;
        break;
      case "restore":
      default:
        components = restorableSelectedComponents.value;
    }

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
    // single selected component
    items.push({
      label: `Copy ${typeDisplayName()} "${
        selectedComponent.value.displayName
      }"`,
      icon: "clipboard-copy",
      onSelect: triggerCopySelection,
      disabled,
    });
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
    // multiple selected components
    items.push({
      label: `Copy ${selectedComponentIds.value.length} Components`,
      icon: "clipboard-copy",
      onSelect: triggerCopySelection,
      disabled,
    });
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
    erasableSelectedComponents.value.length > 0 &&
    erasableSelectedComponents.value.length ===
      selectedComponentsAndChildren.value.length
  ) {
    const label =
      erasableSelectedComponents.value.length === 1
        ? `Erase ${typeDisplayName("erase")} "${
            erasableSelectedComponents.value[0]?.displayName
          }"`
        : `Erase ${erasableSelectedComponents.value.length} ${plur(
            typeDisplayName("erase"),
            erasableSelectedComponents.value.length,
          )}`;

    items.push({
      label,
      icon: "erase",
      onSelect: triggerWipeFromDiagram,
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

function triggerWipeFromDiagram() {
  modelingEventBus.emit("eraseSelection");
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

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, isOpen });
</script>
