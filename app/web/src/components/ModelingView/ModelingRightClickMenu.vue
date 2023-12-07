<template>
  <DropdownMenu ref="contextMenuRef" :items="rightClickMenuItems" />
</template>

<script setup lang="ts">
import {
  DropdownMenuItemObjectDef,
  DropdownMenu,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import plur from "plur";
import { useComponentsStore } from "@/store/components.store";
import { useFixesStore } from "@/store/fixes.store";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const componentsStore = useComponentsStore();
const fixesStore = useFixesStore();

const {
  selectedComponentId,
  selectedComponentIds,
  selectedComponent,
  deletableSelectedComponents,
  restorableSelectedComponents,
  selectedEdgeId,
  selectedEdge,
} = storeToRefs(componentsStore);

function typeDisplayName(action = "delete") {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.nodeType === "component") return "Component";
    else return "Frame";
  } else if (selectedComponentIds.value.length) {
    const components =
      action === "delete"
        ? deletableSelectedComponents.value
        : restorableSelectedComponents.value;

    for (const c of components) {
      if (c.nodeType === "component") return "Component"; // if we have both frames and components, just use the word component
    }

    return "Frame";
  } else {
    return "Component";
  }
}

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = fixesStore.fixesAreInProgress;
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
    if (selectedComponent.value.changeStatus === "deleted") {
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

  if (selectedComponent.value?.resource.data) {
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

const modelingEventBus = componentsStore.eventBus;
function triggerDeleteSelection() {
  modelingEventBus.emit("deleteSelection");
}
function triggerRestoreSelection() {
  modelingEventBus.emit("restoreSelection");
}

function open(e?: MouseEvent, anchorToMouse?: boolean) {
  contextMenuRef.value?.open(e, anchorToMouse);
}
defineExpose({ open });
</script>
