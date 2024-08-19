<template>
  <DropdownMenu
    ref="contextMenuRef"
    :items="rightClickMenuItems"
    variant="editor"
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
// import plur from "plur";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { BindingWithDisplayName, useFuncStore } from "@/store/func/funcs.store";
import { useActionsStore } from "@/store/actions.store";
import { trackEvent } from "@/utils/tracking";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const actionsStore = useActionsStore();

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

// function typeDisplayName(action = "delete") {
//   if (selectedComponentId.value && selectedComponent.value) {
//     if (selectedComponent.value.componentType === ComponentType.Component)
//       return "Component";
//     else return "Frame";
//   } else if (selectedComponentIds.value.length) {
//     let components;
//     switch (action) {
//       case "delete":
//         components = deletableSelectedComponents.value;
//         break;
//       case "erase":
//         components = erasableSelectedComponents.value;
//         break;
//       case "restore":
//       default:
//         components = restorableSelectedComponents.value;
//     }

//     for (const c of components) {
//       if (c.componentType === ComponentType.Component) return "Component"; // if we have both frames and components, just use the word component
//     }

//     return "Frame";
//   } else {
//     return "Component";
//   }
// }

const bindings = computed(() => funcStore.actionBindingsForSelectedComponent);
const canRefresh = computed(
  () =>
    selectedComponent.value?.hasResource &&
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId,
);
const getActionToggleState = (id: string) => {
  if (!selectedComponentId.value) return false;

  const a = actionsStore.listActionsByComponentId
    .get(selectedComponentId.value)
    .find((a) => a.prototypeId === id);
  return !!a;
};

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;

  if (selectedEdgeId.value) {
    // single selected edge
    items.push({
      label: "EDGE",
      header: true,
    });

    if (selectedEdge.value?.changeStatus === "deleted") {
      items.push({
        label: "Restore",
        icon: "trash-restore",
        onSelect: triggerRestoreSelection,
        disabled,
      });
    } else {
      items.push({
        label: "Delete",
        shortcut: "⌫",
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
      });
    }
  } else if (selectedComponentId.value && selectedComponent.value) {
    // single selected component
    items.push({
      label: "COMPONENT",
      header: true,
    });

    if (selectedComponent.value.componentType !== ComponentType.Component) {
      const verb = componentsStore.collapsedComponents.has(
        `g-${selectedComponentId.value}`,
      )
        ? "Expand"
        : "Collapse";
      items.push({
        label: verb,
        icon: componentsStore.collapsedComponents.has(
          `g-${selectedComponentId.value}`,
        )
          ? "chevron--down"
          : "chevron--right",
        onSelect: toggleCollapse,
        disabled,
      });
    }
    items.push({
      label: `Copy`,
      shortcut: "⌘C",
      icon: "clipboard-copy",
      onSelect: triggerCopySelection,
      disabled,
    });
    if (selectedComponent.value.toDelete) {
      items.push({
        label: `Restore`,
        icon: "trash-restore",
        onSelect: triggerRestoreSelection,
        disabled,
      });
    } else {
      items.push({
        label: `Delete`,
        shortcut: "⌫",
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
      });
    }
  } else if (selectedComponentIds.value.length) {
    // multiple selected components
    items.push({
      label: ` ${selectedComponentIds.value.length} COMPONENTS`,
      header: true,
    });

    items.push({
      label: `Copy`,
      shortcut: "⌘C",
      icon: "clipboard-copy",
      onSelect: triggerCopySelection,
      disabled,
    });
    if (deletableSelectedComponents.value.length > 0) {
      items.push({
        label: `Delete`,
        shortcut: "⌫",
        icon: "trash",
        onSelect: triggerDeleteSelection,
        disabled,
      });
    }
    if (restorableSelectedComponents.value.length > 0) {
      items.push({
        label: `Restore`,
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
    items.push({
      label: "Erase",
      shortcut: "⌘E",
      icon: "erase",
      onSelect: triggerWipeFromDiagram,
      disabled,
    });
  }

  if (bindings.value.length > 0 || canRefresh.value) {
    items.push({
      label: "RESOURCE",
      header: true,
    });

    if (canRefresh.value) {
      items.push({
        label: "Refresh",
        shortcut: "R",
        icon: "refresh",
        onSelect: () => {
          // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
          componentsStore.REFRESH_RESOURCE_INFO(selectedComponent.value!.id);
        },
        disabled,
      });
    }

    if (bindings.value.length > 0 && selectedComponentId.value) {
      const submenuItems: DropdownMenuItemObjectDef[] = [];

      bindings.value.forEach((binding: BindingWithDisplayName) => {
        const componentId = selectedComponentId.value as string;

        const action = computed(() => {
          const a = actionsStore.listActionsByComponentId
            .get(componentId)
            .find((a) => a.prototypeId === binding.actionPrototypeId);
          return a;
        });

        submenuItems.push({
          label: binding.displayName,
          toggleIcon: true,
          checked: binding.actionPrototypeId
            ? getActionToggleState(binding.actionPrototypeId)
            : false,
          onSelect: () => {
            if (action.value?.id) {
              actionsStore.CANCEL([action.value.id]);
            } else if (binding.actionPrototypeId) {
              actionsStore.ADD_ACTION(componentId, binding.actionPrototypeId);
            }
          },
        });
      });

      items.push({
        label: "Actions",
        submenuItems,
      });
    }
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

const toggleCollapse = () => {
  const uniqueKey = `g-${selectedComponentId.value}`;
  if (componentsStore.collapsedComponents.has(uniqueKey)) {
    componentsStore.expandComponents(uniqueKey);
    trackEvent("expand-components", {
      source: "context-menu",
      schemaVariantName: selectedComponent.value?.schemaVariantName,
      schemaName: selectedComponent.value?.schemaName,
      hasParent: !!selectedComponent.value?.parentId,
    });
  } else {
    const { position, size } =
      componentsStore.initMinimzedElementPositionAndSize(uniqueKey);
    componentsStore.updateMinimzedElementPositionAndSize({
      uniqueKey,
      position,
      size,
    });
    trackEvent("collapse-components", {
      source: "context-menu",
      schemaVariantName: selectedComponent.value?.schemaVariantName,
      schemaName: selectedComponent.value?.schemaName,
      hasParent: !!selectedComponent.value?.parentId,
    });
  }
};

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, isOpen });
</script>
