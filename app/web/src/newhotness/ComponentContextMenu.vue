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
  ComponentInList,
  EntityKind,
  SchemaVariant,
} from "@/workers/types/entity_kind_types";
import CreateTemplateModal from "@/newhotness/CreateTemplateModal.vue";
import EraseModal from "./EraseModal.vue";
import DeleteModal, { DeleteMode } from "./DeleteModal.vue";
import { useApi, routes } from "./api_composables";
import { assertIsDefined, ExploreContext } from "./types";
import { useComponentDeletion } from "./composables/useComponentDeletion";
import { useComponentUpgrade } from "./composables/useComponentUpgrade";
import { useComponentActions } from "./logic_composables/component_actions";

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
      icon: "trash-restore",
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

  // TODO(nick): add the ability to add and remove components from views.
  // items.push({
  //   icon: "plus",
  //   label: "Add to View",
  //   shortcut: "+",
  //   onSelect: () => componentsAddToView(componentIds.value),
  // });
  // items.push({
  //   icon: "minus",
  //   label: "Remove from View",
  //   shortcut: "-",
  //   onSelect: () => componentsRemoveFromView(componentIds.value),
  // });

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

// TODO(nick): add the ability to add and remove components from views.
// const componentsAddToView = async (componentIds: ComponentId[]) => {
//   close();
// };
// const componentsRemoveFromView = async (componentIds: ComponentId[]) => {
//   close();
// };

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
  // TODO(nick): add the ability to add and remove components from views.
  // componentsAddToView,
  // componentsRemoveFromView,
  componentsUpgrade,
  contextMenuRef,
  componentsStartDelete,
  componentsRestore,
  createTemplateStart,
  focusFirstItem,
});
</script>
