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
    <CreateTemplateModal ref="createTemplateModalRef" />
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
import CreateTemplateModal from "@/newhotness/CreateTemplateModal.vue";
import EraseModal from "./EraseModal.vue";
import DeleteModal, { DeleteMode } from "./DeleteModal.vue";
import { useApi, routes } from "./api_composables";
import { assertIsDefined, ExploreContext } from "./types";
import { useComponentDeletion } from "./composables/useComponentDeletion";
import { useComponentUpgrade } from "./composables/useComponentUpgrade";

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
const componentIds = computed(() => components.value?.map((c) => c.id));

// Use shared deletion composable with view context
const { deleteComponents, eraseComponents, restoreComponents } =
  useComponentDeletion(explore.viewId.value);

// Use shared upgrade composable
const { upgradeComponents } = useComponentUpgrade();

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
    labelAsTooltip: false,
    label: "Erase",
    shortcut: "E",
    icon: "erase" as const,
    iconClass: "text-destructive-300",
    shortcutClass: "border-destructive-200 text-destructive-300",
    onSelect: () => componentsStartErase(components.value),
  };

  // If everything is ghosted, only add the ability to restore/erase and return.
  if (atLeastOneGhostedComponent.value) {
    items.push({
      labelAsTooltip: false,
      label: "Restore",
      shortcut: "R",
      // icon: "trash-restore",
      onSelect: () => componentsRestore(componentIds.value),
    });
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

  // Only enable pinning if we are working with a single component on the grid.
  if (props.onGrid && singleComponent.value) {
    const componentId = singleComponent.value.id;
    items.push({
      labelAsTooltip: false,
      label: "Pin",
      shortcut: "P",
      icon: "pin",
      onSelect: () => emit("pin", componentId),
    });
  }

  items.push({
    labelAsTooltip: false,
    label: "Duplicate",
    shortcut: "D",
    icon: "clipboard-copy",
    onSelect: () => componentsDuplicate(componentIds.value),
  });

  // Can't create template with ghosted components
  if (!atLeastOneGhostedComponent.value) {
    items.push({
      labelAsTooltip: false,
      label: "Create Template",
      shortcut: "T",
      icon: "template-new",
      onSelect: createTemplateStart,
    });
  }

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
        label: "Management Funcs",
        submenuItems: mgmtFuncsSubmenuItems,
        submenuVariant: "contextmenu",
      });
    }
  }

  // multiple components, nothing `toDelete`
  if (components.value.length > 1 && !atLeastOneGhostedComponent.value) {
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
  if (addActionApi.ok(req) && newChangeSetId) {
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

const componentsRestore = async (componentIds: ComponentId[]) => {
  await restoreComponents(componentIds);
};

const eraseComponentIds = ref<ComponentId[] | undefined>(undefined);
const eraseModalRef = ref<InstanceType<typeof EraseModal>>();

const componentsStartErase = (components: ComponentInList[]) => {
  eraseComponentIds.value = componentIds.value;
  eraseModalRef.value?.open(components);
  close();
};
const componentsFinishErase = async () => {
  if (!eraseComponentIds.value || eraseComponentIds.value.length === 0) return;

  const result = await eraseComponents(eraseComponentIds.value);
  if (result.success) {
    eraseModalRef.value?.close();
  }
};

const deleteComponentIds = ref<ComponentId[]>([]);
const deleteModalRef = ref<InstanceType<typeof DeleteModal>>();

const componentsStartDelete = (components: ComponentInList[]) => {
  deleteComponentIds.value = componentIds.value;
  deleteModalRef.value?.open(components);
  close();
};
const componentsFinishDelete = async (mode: DeleteMode) => {
  if (!deleteComponentIds.value || deleteComponentIds.value.length < 1) return;

  const result = await deleteComponents(deleteComponentIds.value, mode);
  if (result.success) {
    deleteModalRef.value?.close();
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

const componentsUpgrade = async (componentIds: ComponentId[]) => {
  await upgradeComponents(componentIds);
};

const createTemplateModalRef = ref<InstanceType<typeof CreateTemplateModal>>();

const createTemplateStart = () => {
  createTemplateModalRef.value?.open(componentIds.value, explore.viewId.value);
  close();
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
  (e: "bulk"): void;
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
  createTemplateStart,
  focusFirstItem,
});
</script>
