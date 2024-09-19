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
import { RouteLocationRaw } from "vue-router";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { BindingWithDisplayName, useFuncStore } from "@/store/func/funcs.store";
import { useActionsStore } from "@/store/actions.store";
import { useComponentAttributesStore } from "@/store/component_attributes.store";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeSetsStore = useChangeSetsStore();
const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const actionsStore = useActionsStore();

const {
  selectedComponentId,
  selectedComponentIds,
  selectedComponent,
  selectedComponents,
  selectedComponentsAndChildren,
  deletableSelectedComponents,
  restorableSelectedComponents,
  erasableSelectedComponents,
  selectedEdgeId,
  selectedEdge,
} = storeToRefs(componentsStore);

const attributesStore = computed(() =>
  selectedComponentId.value
    ? useComponentAttributesStore(selectedComponentId.value)
    : undefined,
);

function typeDisplayName() {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.componentType === ComponentType.Component)
      return "COMPONENT";
    else if (
      selectedComponent.value.componentType ===
      ComponentType.ConfigurationFrameUp
    )
      return "UP FRAME";
    else return "DOWN FRAME";
  } else if (selectedComponentIds.value.length) {
    for (const c of selectedComponents.value) {
      if (c.componentType === ComponentType.Component) return "COMPONENTS"; // if we have both frames and components, just use the word component
    }
    return "FRAMES";
  } else {
    return "COMPONENT";
  }
}

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
      label: typeDisplayName(),
      header: true,
    });

    // checks if this is a collapsed frame
    const collapsed = componentsStore.collapsedComponents.has(
      `g-${selectedComponentId.value}`,
    );

    // rename
    if (!collapsed) {
      items.push({
        label: "Rename",
        shortcut: "N",
        icon: "cursor",
        onSelect: renameComponent,
      });
    }

    // set component type
    {
      const updateComponentType = (componentType: ComponentType) => {
        if (selectedComponentId.value && attributesStore.value) {
          attributesStore.value.SET_COMPONENT_TYPE({
            componentId: selectedComponentId.value,
            componentType,
          });
        }
      };

      const submenuItems: DropdownMenuItemObjectDef[] = [];
      submenuItems.push({
        label: "Component",
        icon: "component",
        checkable: true,
        checked:
          selectedComponent.value.componentType === ComponentType.Component,
        onSelect: () => {
          updateComponentType(ComponentType.Component);
        },
      });
      submenuItems.push({
        label: "Up Frame",
        icon: "frame-up",
        checkable: true,
        checked:
          selectedComponent.value.componentType ===
          ComponentType.ConfigurationFrameUp,
        onSelect: () => {
          updateComponentType(ComponentType.ConfigurationFrameUp);
        },
      });
      submenuItems.push({
        label: "Down Frame",
        icon: "frame-down",
        checkable: true,
        checked:
          selectedComponent.value.componentType ===
          ComponentType.ConfigurationFrameDown,
        onSelect: () => {
          updateComponentType(ComponentType.ConfigurationFrameDown);
        },
      });

      items.push({
        label: "Set Type",
        icon: "component",
        submenuItems,
      });
    }

    // expand and collapse for a single frame
    if (selectedComponent.value.componentType !== ComponentType.Component) {
      const verb = collapsed ? "Expand" : "Collapse";
      items.push({
        label: verb,
        icon: componentsStore.collapsedComponents.has(
          `g-${selectedComponentId.value}`,
        )
          ? "chevron--down"
          : "chevron--right",
        onSelect: menuCollapse,
        disabled,
        shortcut: "\\",
      });
    }

    // copy, restore, delete
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
      label: `${selectedComponentIds.value.length} ${typeDisplayName()}`,
      header: true,
    });

    // expand and collapse for multiple frames
    const collapseOrExpand =
      componentsStore.collapseOrExpandSelectedComponents();
    if (collapseOrExpand) {
      const verb = collapseOrExpand;
      items.push({
        label: verb,
        icon:
          collapseOrExpand === "Collapse" ? "chevron--down" : "chevron--right",
        onSelect: menuCollapse,
        disabled,
        shortcut: "\\",
      });
    }

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
          label: binding.displayName || binding.name,
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
          endLinkTo: {
            name: "workspace-lab-assets",
            query: {
              s: `a_${selectedComponent.value?.schemaVariantId}|f_${binding.funcId}`,
            },
          } as RouteLocationRaw,
          endLinkLabel: "view",
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

function renameComponent() {
  if (selectedComponentId.value) {
    componentsStore.eventBus.emit("rename", selectedComponentId.value);
  }
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

const menuCollapse = () => {
  componentsStore.toggleSelectedCollapse("context-menu");
};

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, isOpen });
</script>
