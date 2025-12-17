<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts">
// Toggle this to true to show debugging info on the menu
export const DEBUG_MODE = false;
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<template>
  <div>
    <DropdownMenu
      ref="contextMenuRef"
      :anchorTo="anchor"
      :items="rightClickMenuItems"
      variant="contextmenu"
      noDefaultClose
      :disableKeyboardControls="!enableKeyboardControls"
      :alignOutsideRightEdge="onGrid"
      :alignOutsideLeftEdge="!onGrid"
      :overlapAnchorOffset="Y_OFFSET"
      :anchorXOffset="4"
      @enterPressedNoSelection="() => emit('edit')"
    />
    <EraseModal ref="eraseModalRef" @confirm="componentsFinishErase" />
    <DeleteModal
      ref="deleteModalRef"
      @delete="(mode) => componentsFinishDelete(mode)"
    />
    <DuplicateComponentsModal
      ref="duplicateComponentsModalRef"
      @confirm="duplicateComponentsFinish"
    />
  </div>
</template>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import { computed, inject, nextTick, ref } from "vue";
import { RouteLocationRaw, useRoute } from "vue-router";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  ComponentInList,
  EntityKind,
  SchemaVariant,
} from "@/workers/types/entity_kind_types";
import EraseModal from "./EraseModal.vue";
import DeleteModal, { DeleteMode } from "./DeleteModal.vue";
import { useApi, routes } from "./api_composables";
import { assertIsDefined, ExploreContext } from "./types";
import { useComponentDeletion } from "./composables/useComponentDeletion";
import { useComponentUpgrade } from "./composables/useComponentUpgrade";
import { useComponentActions } from "./logic_composables/component_actions";
import DuplicateComponentsModal from "./DuplicateComponentsModal.vue";
import {
  availableViewListOptionsForComponentIds,
  useComponentsAndViews,
} from "./logic_composables/view";

const props = defineProps<{
  onGrid?: boolean;
  enableKeyboardControls?: boolean;
  viewListOptions?: { label: string; value: string }[];
  hidePin?: boolean;
  hideBulk?: boolean;
}>();

const route = useRoute();

// This number fixes the Y position to align with the ExploreGridTile
const Y_OFFSET = 4;

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const key = useMakeKey();
const args = useMakeArgs();

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const components = ref<ComponentInList[]>([]);
const componentIds = computed(() => components.value?.map((c) => c.id));

// Use shared deletion composable with view context
const { deleteComponents, eraseComponents, restoreComponents } =
  useComponentDeletion(explore.viewId.value);

// Use shared upgrade composable
const { upgradeComponents } = useComponentUpgrade();

// Use shared composable for components and views
const componentsAndViews = useComponentsAndViews();

const atLeastOneGhostedComponent = computed(() =>
  components.value.some((c) => c.toDelete),
);
const atLeastOneNormalComponent = computed(() =>
  components.value.some((c) => !c.toDelete),
);

// ================================================================================================
// HANDLE SINGLE COMPONENT MENU OPTIONS
const singleComponent = computed(() => {
  if (components.value.length === 1 && components.value[0]) {
    return components.value[0];
  }
  return null;
});

// Use the composable for action functionality
const { actionPrototypeViews, actionByPrototype, toggleActionHandler } =
  useComponentActions(singleComponent);

const schemaVariantId = computed(
  () => singleComponent.value?.schemaVariantId ?? "",
);
const schemaVariantQuery = useQuery<SchemaVariant | null>({
  enabled: () => singleComponent.value !== undefined,
  queryKey: key(EntityKind.SchemaVariant, schemaVariantId),
  queryFn: async () => {
    return await bifrost<SchemaVariant>(
      args(EntityKind.SchemaVariant, singleComponent.value?.schemaVariantId),
    );
  },
});
const managementFunctions = computed(
  () => schemaVariantQuery.data.value?.mgmtFunctions ?? [],
);

// ================================================================================================
// HANDLE VIEW MENU OPTIONS
const availableViewListOptions = computed(() =>
  availableViewListOptionsForComponentIds(
    componentIds.value,
    props.viewListOptions ?? [],
    componentsAndViews,
  ),
);

const removeFromViewTooltip = computed(() => {
  if (availableViewListOptions.value.removeFromView.length > 0)
    return undefined;
  const unprocessedOptions = props.viewListOptions ?? [];
  for (const unprocessedOption of unprocessedOptions) {
    const viewId = unprocessedOption.value;
    for (const componentId of componentIds.value) {
      const soleViewIdForCurrentComponent =
        componentsAndViews.componentsInOnlyOneView.value[componentId];
      if (soleViewIdForCurrentComponent === viewId)
        return "Cannot remove components from their final view. A given component must exist in at least one view.";
    }
  }
  return undefined;
});

// ================================================================================================
// BEGIN CREATING THE MENU OPTIONS
const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  if (DEBUG_MODE) {
    items.push({
      label: `COMPONENTS SELECTED: ${components.value.length}`,
      header: true,
    });
  }

  const eraseMenuItem = {
    labelAsTooltip: false,
    label: "Erase",
    shortcut: "E",
    icon: "erase" as const,
    iconClass: "text-destructive-300",
    shortcutClass: "border-destructive-200 text-destructive-300",
    onSelect: () => componentsStartErase(components.value),
  };

  // If all components are ghosted, only add the ability to restore/erase and return.
  if (atLeastOneGhostedComponent.value && !atLeastOneNormalComponent.value) {
    items.push({
      labelAsTooltip: false,
      label: "Restore",
      shortcut: "R",
      icon: "trash-restore",
      onSelect: () => componentsRestore(componentIds.value),
    });
    items.push(eraseMenuItem);
    return items;
  }

  // If we have a mix of ghosted and normal components, limit available actions
  if (atLeastOneGhostedComponent.value && atLeastOneNormalComponent.value) {
    // Only erase is available for mixed selections
    items.push(eraseMenuItem);
    return items;
  }

  const upgradeableComponents = explore.upgradeableComponents.value;

  // Get only the components that are actually upgradeable
  const upgradeableSelectedComponentIds = componentIds.value.filter((cId) =>
    upgradeableComponents.has(cId),
  );

  if (upgradeableSelectedComponentIds.length > 0) {
    const allUpgradeable =
      upgradeableSelectedComponentIds.length === components.value.length;
    const label = allUpgradeable
      ? "Upgrade"
      : `Upgrade (${upgradeableSelectedComponentIds.length}/${components.value.length})`;

    items.push({
      labelAsTooltip: false,
      label,
      shortcut: "U",
      icon: "bolt-outline",
      disabled: !allUpgradeable,
      onSelect: () => {
        if (allUpgradeable) {
          componentsUpgrade(upgradeableSelectedComponentIds);
        }
      },
    });
  }

  if (singleComponent.value) {
    items.push({
      labelAsTooltip: false,
      label: "Edit",
      shortcut: "⏎",
      icon: "edit2",
      onSelect: () => emit("edit"),
    });
  }

  // Only enable pinning if we are working with a single component on the grid and pin is not hidden.
  if (props.onGrid && singleComponent.value && !props.hidePin) {
    const componentId = singleComponent.value.id;
    items.push({
      labelAsTooltip: false,
      label: "Pin",
      shortcut: "P",
      icon: "pin",
      onSelect: () => {
        emit("pin", componentId);
      },
    });
  }

  items.push({
    labelAsTooltip: false,
    label: "Duplicate",
    shortcut: "D",
    icon: "clipboard-copy",
    onSelect: () => duplicateComponentStart(componentIds.value),
  });

  // can erase so long as you have not selected a view
  items.push(eraseMenuItem);

  items.push({
    labelAsTooltip: false,
    label: "Delete",
    shortcut: "⌫",
    icon: "trash",
    iconClass: "text-destructive-300",
    shortcutClass: "border-destructive-200 text-destructive-300",
    onSelect: () => componentsStartDelete(components.value),
  });

  if (availableViewListOptions.value.addToView.length > 0) {
    const submenuItems: DropdownMenuItemObjectDef[] = [];
    for (const option of availableViewListOptions.value.addToView) {
      submenuItems.push({
        label: option.label,
        onSelect: () => componentsAddToView(option.value, componentIds.value),
      });
    }
    items.push({
      icon: "plus",
      label: "Add to View",
      submenuItems,
      submenuVariant: "contextmenu",
    });
  }

  if (removeFromViewTooltip.value) {
    items.push({
      icon: "minus",
      label: "Remove from View",
      disabled: true,
      showTooltipOnHover: true,
      tooltip: removeFromViewTooltip.value,
    });
  } else if (availableViewListOptions.value.removeFromView.length > 0) {
    const submenuItems: DropdownMenuItemObjectDef[] = [];
    for (const option of availableViewListOptions.value.removeFromView) {
      submenuItems.push({
        label: option.label,
        onSelect: () =>
          componentsRemoveFromView(option.value, componentIds.value),
      });
    }
    items.push({
      icon: "minus",
      label: "Remove from View",
      submenuItems,
      submenuVariant: "contextmenu",
    });
  }

  // Only enable actions if we are working with a single component.
  if (singleComponent.value && singleComponent.value.schemaVariantId) {
    const actionsSubmenuItems: DropdownMenuItemObjectDef[] = [];
    for (const actionPrototype of actionPrototypeViews.value) {
      const existingActionId = actionByPrototype.value[actionPrototype.id]?.id;
      const { handleToggle } = toggleActionHandler(
        actionPrototype,
        () => existingActionId,
      );
      actionsSubmenuItems.push({
        label: actionPrototype.displayName || actionPrototype.name,
        toggleIcon: true,
        checked: existingActionId !== undefined,
        onSelect: handleToggle,
        endLinkTo: {
          name: "workspace-lab-assets",
          query: {
            s: `a_${singleComponent.value?.schemaVariantId}|f_${actionPrototype.funcId}`,
          },
        } as RouteLocationRaw,
        endLinkLabel: "view",
      });
    }

    if (actionsSubmenuItems.length > 0) {
      items.push({
        icon: "bullet-list",
        label: "Actions",
        submenuItems: actionsSubmenuItems,
        submenuVariant: "contextmenu",
      });
    }

    const mgmtFuncsSubmenuItems: DropdownMenuItemObjectDef[] = [];
    for (const mgmtFunction of managementFunctions.value) {
      mgmtFuncsSubmenuItems.push({
        label: mgmtFunction.name,
        onSelect: () => {
          runMgmtFunc(mgmtFunction.id);
        },
      });
    }

    if (mgmtFuncsSubmenuItems.length > 0) {
      items.push({
        icon: "func",
        label: "Management Funcs",
        submenuItems: mgmtFuncsSubmenuItems,
        submenuVariant: "contextmenu",
      });
    }
  }

  // multiple components, nothing `toDelete`
  if (
    components.value.length > 1 &&
    !atLeastOneGhostedComponent.value &&
    !props.hideBulk
  ) {
    items.push({
      label: "Bulk",
      shortcut: "B",
      icon: "edit" as const,
      onSelect: (event: MouseEvent) => {
        emit("bulk");
        event.stopPropagation();
        close();
      },
    });
  }

  return items;
});
// END CREATING THE MENU OPTIONS
// ================================================================================================

const mgmtRunApi = useApi();
const runMgmtFunc = async (funcId: string) => {
  if (!singleComponent.value?.id) return;
  const call = mgmtRunApi.endpoint<{ success: boolean }>(routes.MgmtFuncRun, {
    prototypeId: funcId,
    componentId: singleComponent.value?.id,
    viewId: "DEFAULT",
  });

  const { req, newChangeSetId } = await call.post({});

  if (mgmtRunApi.ok(req) && newChangeSetId) {
    mgmtRunApi.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
        },
      },
      newChangeSetId,
    );
  }
};

const componentsRestore = async (componentIds: ComponentId[]) => {
  await restoreComponents(componentIds);
};

const eraseComponentIds = ref<ComponentId[] | undefined>(undefined);
const eraseModalRef = ref<InstanceType<typeof EraseModal>>();

const componentsStartErase = (components: ComponentInList[]) => {
  eraseComponentIds.value = components.map((c) => c.id);
  eraseModalRef.value?.open(components);
  close();
};
const componentsFinishErase = async () => {
  if (!eraseComponentIds.value || eraseComponentIds.value.length === 0) return;

  const result = await eraseComponents(eraseComponentIds.value);
  if (result.success) {
    eraseModalRef.value?.close();
    emit("finishAction");
  }
};

const deleteComponentIds = ref<ComponentId[]>([]);
const deleteModalRef = ref<InstanceType<typeof DeleteModal>>();

const componentsStartDelete = (components: ComponentInList[]) => {
  const atLeastOneGhostedComponent = components.some((c) => c.toDelete);
  const atLeastOneNormalComponent = components.some((c) => !c.toDelete);
  if (atLeastOneGhostedComponent && atLeastOneNormalComponent) return;
  if (components.length < 1) return;
  deleteComponentIds.value = components.map((c) => c.id);
  deleteModalRef.value?.open(components);
  close();
};
const componentsFinishDelete = async (mode: DeleteMode) => {
  if (!deleteComponentIds.value || deleteComponentIds.value.length < 1) return;

  const result = await deleteComponents(deleteComponentIds.value, mode);
  if (result.success) {
    deleteModalRef.value?.close();
    emit("finishAction");
  }
};

const duplicateComponentIds = ref<ComponentId[]>([]);
const duplicateComponentsModalRef =
  ref<InstanceType<typeof DuplicateComponentsModal>>();
const isDuplicating = ref(false);

const duplicateComponentStart = (componentIds: ComponentId[]) => {
  duplicateComponentIds.value = componentIds;
  duplicateComponentsModalRef.value?.open(componentIds, explore.viewId.value);
  close();
};

const duplicateComponentsFinish = async (name: string) => {
  if (isDuplicating.value) return;
  if (!duplicateComponentIds.value || duplicateComponentIds.value.length < 1)
    return;

  isDuplicating.value = true;
  try {
    const result = await duplicateComponents(duplicateComponentIds.value, name);
    if (result.success) {
      duplicateComponentsModalRef.value?.close();
      emit("finishAction");
    }
  } finally {
    isDuplicating.value = false;
  }
};
const duplicateComponentApi = useApi();

const duplicateComponents = async (
  componentIds: ComponentId[],
  name: string,
) => {
  const call = duplicateComponentApi.endpoint(routes.DuplicateComponents, {
    viewId: explore.viewId.value,
  });
  emit("clearSelected");
  const { req, newChangeSetId } = await call.post({
    components: componentIds,
    name,
  });

  if (duplicateComponentApi.ok(req) && newChangeSetId) {
    duplicateComponentApi.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
        },
      },
      newChangeSetId,
    );
  }
  return { success: duplicateComponentApi.ok(req), newChangeSetId };
};

const addToViewApi = useApi();
const removeFromViewApi = useApi();
const componentsAddToView = async (
  viewId: string,
  componentIds: ComponentId[],
) => {
  const call = addToViewApi.endpoint(routes.ViewAddComponents, {
    viewId,
  });
  close();
  const { req, newChangeSetId } = await call.post({
    componentIds,
  });

  if (addToViewApi.ok(req) && newChangeSetId) {
    addToViewApi.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
        },
      },
      newChangeSetId,
    );
  }
};
const componentsRemoveFromView = async (
  viewId: string,
  componentIds: ComponentId[],
) => {
  const call = removeFromViewApi.endpoint(routes.ViewEraseComponents, {
    viewId,
  });
  close();
  const { req, newChangeSetId } = await call.delete({
    componentIds,
  });

  if (removeFromViewApi.ok(req) && newChangeSetId) {
    removeFromViewApi.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: route.params.workspacePk,
          changeSetId: newChangeSetId,
        },
      },
      newChangeSetId,
    );
  }
};

const componentsUpgrade = async (componentIds: ComponentId[]) => {
  await upgradeComponents([...componentIds]);
  close();
};

// eslint-disable-next-line @typescript-eslint/ban-types
const anchor = ref<HTMLElement | object | undefined>(undefined);

function open(
  anchorTo: HTMLElement | object,
  componentsForMenu: ComponentInList[],
) {
  const oldAnchor = anchor.value;
  anchor.value = anchorTo;
  components.value = componentsForMenu;
  nextTick(() => {
    if (oldAnchor !== anchor.value || !contextMenuRef.value?.isOpen) {
      contextMenuRef.value?.open();
    }
  });
}

function setSelectedComponents(componentsForMenu: ComponentInList[]) {
  components.value = componentsForMenu;
}

function close() {
  components.value = [];
  contextMenuRef.value?.forceClose();
}

const focusFirstItem = (onlyIfNoFocus = false) => {
  contextMenuRef.value?.focusFirstItem(onlyIfNoFocus);
};

const isOpen = computed(() => contextMenuRef.value?.isOpen);

const emit = defineEmits<{
  (e: "edit"): void;
  (e: "clearSelected"): void;
  (e: "pin", componentId: ComponentId): void;
  (e: "bulk"): void;
  (e: "finishAction"): void;
}>();

defineExpose({
  open,
  close,
  isOpen,
  componentsStartErase,
  duplicateComponentStart,
  componentsAddToView,
  componentsRemoveFromView,
  componentsUpgrade,
  contextMenuRef,
  componentsStartDelete,
  componentsRestore,
  focusFirstItem,
  setSelectedComponents,
});
</script>
