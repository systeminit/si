<template>
  <div>
    <DropdownMenu
      v-if="selectedEdge"
      ref="contextMenuRef"
      :items="rightClickMenuItemsEdge"
      variant="editor"
    />
    <DropdownMenu
      v-else
      ref="contextMenuRef"
      :items="rightClickMenuItems"
      variant="editor"
    />
    <Modal
      ref="modalRef"
      saveLabel="Create"
      size="sm"
      title="Create View"
      type="save"
      @save="create"
    >
      <VormInput
        ref="labelRef"
        v-model="viewName"
        label="View Name"
        required
        @enterPressed="create"
      />
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
  Modal,
  VormInput,
} from "@si/vue-lib/design-system";
import { storeToRefs } from "pinia";
import { computed, ref } from "vue";
import { RouteLocationRaw } from "vue-router";
import { IRect } from "konva/lib/types";
import { ComponentType } from "@/api/sdf/dal/schema";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useStatusStore } from "@/store/status.store";
import {
  BindingWithDisplayName,
  useFuncStore,
  MgmtPrototype,
} from "@/store/func/funcs.store";
import { useActionsStore } from "@/store/actions.store";
import { useViewsStore } from "@/store/views.store";
import { ComponentId } from "@/api/sdf/dal/component";
import { ViewId } from "@/api/sdf/dal/views";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import {
  DiagramGroupData,
  DiagramNodeData,
  DiagramNodeDef,
  DiagramViewData,
} from "../ModelingDiagram/diagram_types";
import { FindChildrenByBoundingBox } from "../ModelingDiagram/utils/childrenByBoundingBox";

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const changeSetsStore = useChangeSetsStore();
const statusStore = useStatusStore();
const componentsStore = useComponentsStore();
const funcStore = useFuncStore();
const actionsStore = useActionsStore();
const viewsStore = useViewsStore();
const featureFlagsStore = useFeatureFlagsStore();

const {
  selectedComponentId,
  selectedComponentIds,
  selectedComponent,
  selectedComponents,
  selectedComponentsAndChildren,
  deletableSelectedComponents,
  restorableSelectedComponents,
  erasableSelectedComponents,
  selectedEdge,
} = storeToRefs(viewsStore);

function typeDisplayName() {
  if (selectedComponentId.value && selectedComponent.value) {
    if (selectedComponent.value.def.componentType === ComponentType.Component)
      return "COMPONENT";
    else if (
      selectedComponent.value.def.componentType ===
      ComponentType.ConfigurationFrameUp
    )
      return "UP FRAME";
    else if (
      selectedComponent.value.def.componentType ===
      ComponentType.ConfigurationFrameDown
    )
      return "DOWN FRAME";
    else if (selectedComponent.value.def.componentType === ComponentType.View)
      return "VIEW";
    else return "ASSET";
  } else if (selectedComponentIds.value.length) {
    for (const c of selectedComponents.value) {
      if (c.def.componentType === ComponentType.Component) return "COMPONENTS"; // if we have both frames and components, just use the word component
    }
    return "ASSETS";
  } else {
    return "ASSET";
  }
}

const bindings = computed(() => funcStore.actionBindingsForSelectedComponent);
const canRefresh = computed(
  () =>
    selectedComponent.value?.def &&
    "hasResource" in selectedComponent.value.def &&
    selectedComponent.value.def.hasResource &&
    changeSetsStore.selectedChangeSetId === changeSetsStore.headChangeSetId,
);
const getActionToggleState = (id: string) => {
  if (!selectedComponentId.value) return false;

  const a = actionsStore.listActionsByComponentId
    .get(selectedComponentId.value)
    .find((a) => a.prototypeId === id);
  return !!a;
};

const isLoading = computed(
  () =>
    selectedComponent.value?.def &&
    statusStore.componentIsLoading(selectedComponent.value?.def.id),
);

const removeFromView = () => {
  viewsStore.removeSelectedViewComponentFromCurrentView();
};

const viewsSubitems = (add: (viewId: ViewId) => void) => {
  // dont show the view you're in b/c you cannot copy or move things to it
  return viewsStore.viewList
    .filter((v) => v.id !== viewsStore.selectedViewId)
    .map((v) => {
      return {
        label: v.name,
        onSelect: () => add(v.id),
      };
    });
};

const viewAdd = (remove: boolean) => {
  return (viewId: ViewId) => {
    const components: Record<ComponentId, IRect> = {};
    selectedComponents.value
      .filter(
        (c): c is DiagramGroupData | DiagramNodeData =>
          c.def.componentType !== ComponentType.View,
      )
      .forEach((c) => {
        const geo = c.def.isGroup
          ? viewsStore.groups[c.def.id]
          : viewsStore.components[c.def.id];
        if (geo) components[c.def.id] = geo;
      });

    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    viewsStore.ADD_TO(viewsStore.selectedViewId!, components, viewId, remove);
  };
};

const rightClickMenuItemsEdge = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;
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
  return items;
});

const anyViews = computed(() =>
  selectedComponents.value.some((c) => c instanceof DiagramViewData),
);

const modalRef = ref<InstanceType<typeof Modal>>();
const labelRef = ref<InstanceType<typeof VormInput>>();
const viewName = ref("");
const newView = () => {
  modalRef.value?.open();
};

const convertToView = async () => {
  if (!viewsStore.selectedViewId) return;
  if (!viewsStore.selectedComponent) return;
  const children = FindChildrenByBoundingBox(
    viewsStore.selectedComponent as DiagramNodeData | DiagramGroupData,
    true,
  );
  const child_ids = children.map((c) => c.def.id);
  if (viewsStore.selectedComponentId) {
    await viewsStore.CONVERT_TO_VIEW(
      viewsStore.selectedViewId,
      viewsStore.selectedComponentId,
      child_ids,
    );
  }
};

const create = async () => {
  if (!viewsStore.selectedViewId) return;
  if (!viewName.value) {
    labelRef.value?.setError("Name is required");
  } else {
    const components: Record<ComponentId, IRect> = {};
    selectedComponents.value.forEach((component) => {
      const geo =
        component.def.componentType === ComponentType.Component
          ? viewsStore.components[component.def.id]
          : viewsStore.groups[component.def.id];
      if (geo) components[component.def.id] = geo;
    });
    const resp = await viewsStore.CREATE_VIEW_AND_MOVE(
      viewName.value,
      viewsStore.selectedViewId,
      components,
    );
    if (resp.result.success) {
      modalRef.value?.close();

      viewName.value = "";
    } else if (resp.result.statusCode === 409) {
      labelRef.value?.setError(
        `${viewName.value} is already in use. Please choose another name`,
      );
    }
  }
};

/**
 * HERE IS THE APPROACH IN GENERAL
 * Make sure every "action" (i.e. onSelect) operates on the whole list of selectedComponents
 * Unless it is disallowed from doing so
 *
 * Don't duplicate `items.push`, only add a thing one time, and focus on the conditions by which it should be added
 */
const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];
  const disabled = false;

  items.push({
    label: "VIEWS",
    header: true,
  });

  // you can do these operations no matter how many elements selected
  if (!anyViews.value) {
    items.push({
      label: "Move to",
      icon: "arrows-out",
      submenuItems: viewsSubitems(viewAdd(true)).concat({
        label: "Create new View ...",
        onSelect: newView,
      }),
    });
    const copyToSubmenuItems = viewsSubitems(viewAdd(false));
    if (copyToSubmenuItems.length > 0) {
      items.push({
        label: "Copy to",
        icon: "clipboard-copy",
        submenuItems: copyToSubmenuItems,
      });
    }
  }
  items.push({
    label: "Convert to View",
    icon: "create",
    onSelect: convertToView,
  });
  items.push({
    label: "Remove",
    icon: "x-circle",
    onSelect: removeFromView,
  });

  // if you've selected a view, you can't do anything else
  if (anyViews.value) return items;

  items.push({
    label: typeDisplayName(),
    header: true,
  });

  // if only one element you can do these operations
  if (selectedComponentId.value && selectedComponent.value) {
    items.push({
      label: "Rename",
      shortcut: "N",
      icon: "cursor",
      onSelect: renameComponent,
    });
    if (featureFlagsStore.FLOATING_CONNECTION_MENU) {
      items.push({
        label: "Connections",
        shortcut: "C",
        icon: "socket",
        onSelect: openConnectionsMenu,
      });
    }
    if (featureFlagsStore.AUTOCONNECT) {
      items.push({
        label: "Auto Connect",
        shortcut: "A",
        icon: "output-socket",
        onSelect: autoConnectComponent,
      });
    }
  }

  // management funcs for a single selected component
  // check if the component is currently updating, as if so we don't
  // want to let the user dispatch a management function
  if (funcStore.managementFunctionsForSelectedComponent.length > 0) {
    const submenuItems: DropdownMenuItemObjectDef[] = [];
    if (!isLoading.value) {
      funcStore.managementFunctionsForSelectedComponent.forEach((fn) => {
        submenuItems.push({
          label: fn.label,
          icon: "play",
          onSelect: () => {
            runManagementFunc(fn);
          },
        });
      });
    } else {
      submenuItems.push({
        label: "Updating...",
        header: true,
      });
    }

    items.push({
      label: "Management",
      icon: "func",
      submenuItems,
    });
  }

  // you copy, restore, delete, template
  items.push({
    label: `Copy`,
    shortcut: "⌘C",
    icon: "clipboard-copy",
    onSelect: triggerCopySelection,
    disabled,
  });
  if (
    restorableSelectedComponents.value.length > 0 &&
    restorableSelectedComponents.value.length ===
      selectedComponentsAndChildren.value.length
  ) {
    items.push({
      label: `Restore`,
      icon: "trash-restore",
      onSelect: triggerRestoreSelection,
      disabled,
    });
  } else if (
    deletableSelectedComponents.value.length > 0 &&
    deletableSelectedComponents.value.length ===
      selectedComponentsAndChildren.value.length
  ) {
    items.push({
      label: `Delete`,
      shortcut: "⌫",
      icon: "trash",
      onSelect: triggerDeleteSelection,
      disabled,
    });
  }
  if (restorableSelectedComponents.value.length === 0) {
    items.push({
      label: `Create Template`,
      shortcut: "T",
      icon: "tools",
      onSelect: triggerTemplateFromSelection,
      disabled,
    });
  }

  // can erase so long as you have not selected a view
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

  // can only refresh a single component
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
          if (selectedComponent.value)
            componentsStore.REFRESH_RESOURCE_INFO(
              selectedComponent.value.def.id,
            );
        },
        disabled,
      });
    }

    // actions limited to a single component
    if (bindings.value.length > 0 && selectedComponentId.value) {
      const def = selectedComponent.value?.def as DiagramNodeDef;
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
              s: `a_${def.schemaVariantId}|f_${binding.funcId}`,
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

const runManagementFunc = async (prototype: MgmtPrototype) => {
  if (!selectedComponent.value) return;
  if (!viewsStore.selectedViewId) return;

  await funcStore.RUN_MGMT_PROTOTYPE(
    prototype.managementPrototypeId,
    selectedComponent.value.def.id,
    viewsStore.selectedViewId,
  );
};

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

function triggerTemplateFromSelection() {
  modelingEventBus.emit("templateFromSelection");
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

function openConnectionsMenu() {
  componentsStore.eventBus.emit("openConnectionsMenu", {
    aDirection: undefined,
    A: {
      componentId: selectedComponentId.value ?? undefined,
      socketId: undefined,
    },
    B: {
      componentId: undefined,
      socketId: undefined,
    },
  });
}

function autoConnectComponent() {
  if (selectedComponentId.value) {
    componentsStore.AUTOCONNECT_COMPONENT(selectedComponentId.value);
  }
}

const elementPos = ref<{ x: number; y: number } | null>(null);

function open(
  e: MouseEvent,
  anchorToMouse: boolean,
  elementPosition?: { x: number; y: number },
) {
  if (elementPosition) elementPos.value = elementPosition;
  if (
    selectedEdge.value &&
    featureFlagsStore.SIMPLE_SOCKET_UI &&
    !selectedEdge.value.isManagement
  )
    return; // for now the right click is disabled on edges in the simple socket ui
  contextMenuRef.value?.open(e, anchorToMouse);
}

function close() {
  contextMenuRef.value?.close();
}

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, close, isOpen });
</script>
