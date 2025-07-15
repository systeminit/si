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
      @enterPressedNoSelection="emit('edit')"
    />
    <EraseModal ref="eraseModalRef" @confirm="componentsFinishErase" />
    <DeleteModal
      ref="deleteModalRef"
      @delete="(mode) => componentsFinishDelete(mode)"
    />
  </div>
</template>

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
  BifrostActionViewList,
  ActionPrototypeViewList,
  ComponentInList,
  EntityKind,
  SchemaVariant,
} from "@/workers/types/entity_kind_types";
import { ActionId, ActionPrototypeId } from "@/api/sdf/dal/action";
import EraseModal from "./EraseModal.vue";
import DeleteModal, { DeleteMode } from "./DeleteModal.vue";
import { useApi, routes } from "./api_composables";
import { assertIsDefined, ExploreContext } from "./types";

const props = defineProps<{
  onGrid?: boolean;
  enableKeyboardControls?: boolean;
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

const atLeastOneGhostedComponent = computed(() =>
  components.value.some((c) => c.toDelete),
);
const atLeastOneNormalComponent = computed(() =>
  components.value.some((c) => !c.toDelete),
);

// ================================================================================================
// These values are for "single component" functionality.
const singleComponent = computed(() =>
  components.value.length === 1 ? components.value[0] : undefined,
);
const schemaVariantId = computed(
  () => singleComponent.value?.schemaVariantId ?? "",
);
const actionPrototypes = computed(
  () => actionPrototypesQuery.data.value?.actionPrototypes ?? [],
);
const actionPrototypesQuery = useQuery<ActionPrototypeViewList | null>({
  enabled: () => schemaVariantId.value !== "",
  queryKey: key(EntityKind.ActionPrototypeViewList, schemaVariantId),
  queryFn: async () =>
    await bifrost<ActionPrototypeViewList>(
      args(EntityKind.ActionPrototypeViewList, schemaVariantId.value),
    ),
});
const actionsQuery = useQuery<BifrostActionViewList | null>({
  enabled: () => singleComponent.value !== undefined,
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionByPrototype = computed(() => {
  if (!singleComponent.value) return {};
  if (!actionsQuery.data.value?.actions) return {};
  if (actionsQuery.data.value.actions.length < 1) return {};

  const result: Record<ActionPrototypeId, ActionId> = {};
  for (const action of actionsQuery.data.value.actions) {
    if (action.componentId === singleComponent.value.id) {
      // NOTE(nick): this assumes that there can be one action for a given prototype and component.
      // As of the time of writing, this is true, but multiple actions per prototype and component
      // aren't disallowed from the underlying graph's perspective. Theorhetically, you could
      // enqueue two refreshes back-to-back. What then? I don't think we'll expose an interface to
      // do that for awhile, so this should be sufficient.
      result[action.prototypeId] = action.id;
    }
  }
  return result;
});
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

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  // If we are dealing with both ghosted and regular components (which should not be possible),
  // then return helper text as a failsafe.
  // TODO(Wendy) - fix how this displays to look nicer!
  if (atLeastOneGhostedComponent.value && atLeastOneNormalComponent.value) {
    items.push({
      label: "No options available for both",
      disabled: true,
    });
    items.push({
      label: "ghosted and regular components",
      disabled: true,
    });
    return items;
  }

  const eraseMenuItem = {
    labelAsTooltip: true,
    label: "Erase",
    shortcut: "E",
    icon: "erase" as const,
    onSelect: () => componentsStartErase(components.value),
  };

  // If everything is ghosted, only add the ability to restore/erase and return.
  if (atLeastOneGhostedComponent.value) {
    items.push({
      labelAsTooltip: true,
      label: "Restore",
      shortcut: "R",
      icon: "trash-restore",
      onSelect: () => componentsRestore(components.value.map((c) => c.id)),
    });
    items.push(eraseMenuItem);
    return items;
  }

  if (singleComponent.value) {
    items.push({
      labelAsTooltip: true,
      label: "Edit",
      shortcut: "Enter",
      icon: "edit2",
      onSelect: () => emit("edit"),
    });
  }

  // can erase so long as you have not selected a view
  items.push(eraseMenuItem);

  items.push({
    labelAsTooltip: true,
    label: "Delete",
    shortcut: "⌫",
    icon: "trash",
    onSelect: () => componentsStartDelete(components.value),
  });

  items.push({
    labelAsTooltip: true,
    label: "Duplicate",
    shortcut: "D",
    icon: "clipboard-copy",
    onSelect: () => componentsDuplicate(components.value.map((c) => c.id)),
  });

  // Only enable pinning if we are working with a single component on the grid.
  if (props.onGrid && singleComponent.value) {
    const componentId = singleComponent.value.id;
    items.push({
      labelAsTooltip: true,
      label: "Pin",
      shortcut: "P",
      icon: "pin",
      onSelect: () => emit("pin", componentId),
    });
  }

  const upgradeableComponents = explore.upgradeableComponents.value;

  // Get only the components that are actually upgradeable
  const upgradeableSelectedComponents = components.value.filter((c) =>
    upgradeableComponents.has(c.id),
  );

  if (upgradeableSelectedComponents.length > 0) {
    const allUpgradeable =
      upgradeableSelectedComponents.length === components.value.length;
    const label = allUpgradeable
      ? "Upgrade"
      : `Upgrade (${upgradeableSelectedComponents.length}/${components.value.length})`;

    items.push({
      labelAsTooltip: true,
      label,
      shortcut: "U",
      icon: "bolt-outline",
      disabled: !allUpgradeable,
      onSelect: () => {
        if (allUpgradeable) {
          componentsUpgrade(upgradeableSelectedComponents.map((c) => c.id));
        }
      },
    });
  }

  // Only enable actions if we are working with a single component.
  if (singleComponent.value && schemaVariantId.value) {
    const componentId = singleComponent.value.id;

    const actionsSubmenuItems: DropdownMenuItemObjectDef[] = [];
    for (const actionPrototype of actionPrototypes.value) {
      const existingActionId = actionByPrototype.value[actionPrototype.id];
      actionsSubmenuItems.push({
        label: actionPrototype.displayName || actionPrototype.name,
        toggleIcon: true,
        checked: existingActionId !== undefined,
        onSelect: () => {
          if (existingActionId) {
            removeAction(existingActionId);
          } else {
            addAction(componentId, actionPrototype.id);
          }
        },
        endLinkTo: {
          name: "workspace-lab-assets",
          query: {
            s: `a_${schemaVariantId.value}|f_${actionPrototype.funcId}`,
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
        label: "Mmgt Funcs",
        submenuItems: mgmtFuncsSubmenuItems,
        submenuVariant: "contextmenu",
      });
    }
  }

  return items;
});

const addActionApi = useApi();
const removeActionApi = useApi();
const addAction = async (
  componentId: ComponentId,
  actionPrototypeId: ActionPrototypeId,
) => {
  const call = addActionApi.endpoint(routes.ActionAdd);

  const { req, newChangeSetId } = await call.post<{
    componentId: string;
    prototypeId: string;
  }>({
    componentId,
    prototypeId: actionPrototypeId,
  });
  if (restoreApi.ok(req) && newChangeSetId) {
    addActionApi.navigateToNewChangeSet(
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
const removeAction = (actionId: ActionId) => {
  const call = removeActionApi.endpoint(routes.ActionCancel, {
    id: actionId,
  });

  // This route can mutate head, so we do not need to handle new change set semantics.
  call.put({});
};

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

const restoreApi = useApi();
const componentsRestore = async (componentIds: ComponentId[]) => {
  const call = restoreApi.endpoint(routes.RestoreComponents);
  const { req, newChangeSetId } = await call.put({
    componentIds,
  });
  if (restoreApi.ok(req) && newChangeSetId) {
    restoreApi.navigateToNewChangeSet(
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

const eraseApi = useApi();
const eraseComponentIds = ref<ComponentId[] | undefined>(undefined);
const eraseModalRef = ref<InstanceType<typeof EraseModal>>();

const componentsStartErase = (components: ComponentInList[]) => {
  const componentIds = components.map((component) => component.id);
  eraseComponentIds.value = componentIds;
  eraseModalRef.value?.open(components);
  close();
};
const componentsFinishErase = async () => {
  if (!eraseComponentIds.value || eraseComponentIds.value.length === 0) return;

  const call = eraseApi.endpoint(routes.DeleteComponents);
  const { req, newChangeSetId } = await call.delete({
    componentIds: eraseComponentIds.value,
    forceErase: true,
  });

  eraseModalRef.value?.close();

  if (eraseApi.ok(req)) {
    if (newChangeSetId) {
      eraseApi.navigateToNewChangeSet(
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
  }
};

const deleteDeleteApi = useApi();
const deleteEraseFromViewApi = useApi();
const deleteComponentIds = ref<ComponentId[]>([]);
const deleteModalRef = ref<InstanceType<typeof DeleteModal>>();

const componentsStartDelete = (components: ComponentInList[]) => {
  deleteComponentIds.value = components.map((c) => c.id);
  deleteModalRef.value?.open(components);
  close();
};
const componentsFinishDelete = async (mode: DeleteMode) => {
  if (!deleteComponentIds.value || deleteComponentIds.value.length < 1) return;

  if (mode === DeleteMode.Delete) {
    const call = deleteDeleteApi.endpoint(routes.DeleteComponents);
    const { req, newChangeSetId } = await call.delete({
      componentIds: deleteComponentIds.value,
      forceErase: false,
    });
    if (deleteDeleteApi.ok(req)) {
      deleteModalRef.value?.close();
      if (newChangeSetId) {
        deleteDeleteApi.navigateToNewChangeSet(
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
    }
  } else {
    const call = deleteEraseFromViewApi.endpoint(
      routes.EraseComponentsFromView,
      { viewId: explore.viewId.value },
    );
    const { req, newChangeSetId } = await call.delete({
      componentIds: deleteComponentIds.value,
    });
    if (deleteEraseFromViewApi.ok(req)) {
      deleteModalRef.value?.close();
      if (newChangeSetId) {
        deleteEraseFromViewApi.navigateToNewChangeSet(
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
    }
  }
};

const duplicateComponentApi = useApi();
const componentsDuplicate = async (componentIds: ComponentId[]) => {
  const call = duplicateComponentApi.endpoint(routes.DuplicateComponents, {
    viewId: explore.viewId.value,
  });
  emit("clearSelected");
  const { req, newChangeSetId } = await call.post({
    components: componentIds,
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
};

const upgradeComponentApi = useApi();
const componentsUpgrade = async (componentIds: ComponentId[]) => {
  const call = upgradeComponentApi.endpoint(routes.UpgradeComponents);
  const { req, newChangeSetId } = await call.post({
    componentIds,
  });
  if (upgradeComponentApi.ok(req) && newChangeSetId) {
    upgradeComponentApi.navigateToNewChangeSet(
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

// eslint-disable-next-line @typescript-eslint/ban-types
const anchor = ref<Object | undefined>(undefined);

function open(
  // eslint-disable-next-line @typescript-eslint/ban-types
  anchorTo: Object,
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
}>();

defineExpose({
  open,
  close,
  isOpen,
  componentsStartErase,
  componentsDuplicate,
  componentsUpgrade,
  contextMenuRef,
  componentsStartDelete,
  componentsRestore,
  focusFirstItem,
});
</script>
