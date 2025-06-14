<template>
  <div>
    <!-- TODO(WENDY) - we might want keyboard controls back in this DropdownMenu at some point? -->
    <!-- for now they are disabled to avoid conflicts with the keyboard controls in Explore! -->
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
import * as _ from "lodash-es";
import {
  DropdownMenu,
  DropdownMenuItemObjectDef,
} from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import { computed, inject, nextTick, ref } from "vue";
import { RouteLocationRaw } from "vue-router";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { ComponentId } from "@/api/sdf/dal/component";
import {
  BifrostActionViewList,
  ActionPrototypeViewList,
  BifrostComponentInList,
  EntityKind,
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

// This number fixes the Y position to align with the ComponentGridTile
const Y_OFFSET = 4;

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const key = useMakeKey();
const args = useMakeArgs();

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

const components = ref<BifrostComponentInList[]>([]);

const atLeastOneGhostedComponent = computed(() =>
  components.value.some((c) => c.toDelete),
);
const atLeastOneNormalComponent = computed(() =>
  components.value.some((c) => !c.toDelete),
);

// ================================================================================================
// These values are for "single component" functionality.
const component = computed(() =>
  components.value.length === 1 ? components.value[0] : undefined,
);
const schemaVariantId = computed(
  () => component.value?.schemaVariantId.id ?? "",
);
const actionPrototypes = computed(
  () => actionPrototypesQuery.data.value?.actionPrototypes ?? [],
);
const actionPrototypesQuery = useQuery<ActionPrototypeViewList | null>({
  enabled: schemaVariantId.value !== "",
  queryKey: key(EntityKind.ActionPrototypeViewList, schemaVariantId),
  queryFn: async () =>
    await bifrost<ActionPrototypeViewList>(
      args(EntityKind.ActionPrototypeViewList, schemaVariantId.value),
    ),
});
const actionsQuery = useQuery<BifrostActionViewList | null>({
  enabled: component.value !== undefined,
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const actionByPrototype = computed(() => {
  if (!component.value) return {};
  if (!actionsQuery.data.value?.actions) return {};
  if (actionsQuery.data.value.actions.length < 1) return {};

  const result: Record<ActionPrototypeId, ActionId> = {};
  for (const action of actionsQuery.data.value.actions) {
    if (action.componentId === component.value.id) {
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
// ================================================================================================

const addActionApi = useApi();
const removeActionApi = useApi();

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  // If we are dealing with both ghosted and regular components (which should not be possible),
  // then return helper text as a failsafe.
  if (atLeastOneGhostedComponent.value && atLeastOneNormalComponent.value) {
    items.push({
      label: "No options available for both ghosted and regular components",
      disabled: true,
    });
    return items;
  }

  // If everything is ghosted, only add the ability to restore and return.
  if (atLeastOneGhostedComponent.value) {
    items.push({
      label: "Restore",
      shortcut: "R",
      icon: "trash-restore",
      onSelect: () => componentsRestore(components.value.map((c) => c.id)),
    });
    return items;
  }

  items.push({
    label: "Edit",
    shortcut: "Enter",
    icon: "edit2",
    onSelect: () => emit("edit"),
  });

  // can erase so long as you have not selected a view
  items.push({
    label: "Erase",
    shortcut: "E",
    icon: "erase",
    onSelect: () => componentsStartErase(components.value.map((c) => c.id)),
  });

  items.push({
    label: "Delete",
    shortcut: "⌫",
    icon: "trash",
    onSelect: () => componentsStartDelete(components.value),
  });

  items.push({
    label: "Duplicate",
    shortcut: "⌘D",
    icon: "clipboard-copy",
    onSelect: () => componentDuplicate(components.value.map((c) => c.id)),
  });

  if (component.value?.canBeUpgraded) {
    items.push({
      label: "Upgrade",
      shortcut: "U",
      icon: "bolt-outline",
      onSelect: () => componentUpgrade(components.value.map((c) => c.id)),
    });
  }

  // Only enable actions if we are working with a single component.
  if (component.value && schemaVariantId.value) {
    const componentId = component.value.id;

    const submenuItems: DropdownMenuItemObjectDef[] = [];

    for (const actionPrototype of actionPrototypes.value) {
      const existingActionId = actionByPrototype.value[actionPrototype.id];
      submenuItems.push({
        label: actionPrototype.displayName || actionPrototype.name,
        toggleIcon: true,
        checked: existingActionId !== undefined,
        onSelect: () => {
          if (existingActionId) {
            const call = removeActionApi.endpoint(routes.ActionCancel, {
              id: existingActionId,
            });

            // TODO(nick): I am not sure that this is needed?
            removeActionApi.setWatchFn(() => existingActionId);

            call.put({});
          } else {
            const call = addActionApi.endpoint(routes.ActionAdd);

            // TODO(nick): I am not sure that this is needed?
            addActionApi.setWatchFn(() => existingActionId);

            call.post<{
              componentId: string;
              prototypeId: string;
            }>({
              componentId,
              prototypeId: actionPrototype.id,
            });
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

    if (submenuItems.length > 0) {
      items.push({
        icon: "bullet-list",
        label: "Actions",
        submenuItems,
        submenuVariant: "contextmenu",
      });
    }
  }

  return items;
});

const restoreApi = useApi();
const componentsRestore = async (componentIds: ComponentId[]) => {
  const call = restoreApi.endpoint(routes.RestoreComponents);
  await call.put({
    componentIds,
  });
};

const eraseApi = useApi();
const eraseComponentIds = ref<ComponentId[] | undefined>(undefined);
const eraseModalRef = ref<InstanceType<typeof EraseModal>>();

const componentsStartErase = (componentIds: ComponentId[]) => {
  eraseComponentIds.value = componentIds;
  eraseModalRef.value?.open();
  close();
};
const componentsFinishErase = async () => {
  if (!eraseComponentIds.value || eraseComponentIds.value.length === 0) return;

  const call = eraseApi.endpoint(routes.DeleteComponents);
  const { req } = await call.delete({
    componentIds: eraseComponentIds.value,
    forceErase: true,
  });

  if (eraseApi.ok(req)) {
    eraseModalRef.value?.close();
  }
};

const deleteDeleteApi = useApi();
const deleteEraseFromViewApi = useApi();
const deleteComponentIds = ref<ComponentId[]>([]);
const deleteModalRef = ref<InstanceType<typeof DeleteModal>>();

const componentsStartDelete = (components: BifrostComponentInList[]) => {
  deleteComponentIds.value = components.map((c) => c.id);
  deleteModalRef.value?.open(components);
  close();
};
const componentsFinishDelete = async (mode: DeleteMode) => {
  if (!deleteComponentIds.value || deleteComponentIds.value.length < 1) return;

  if (mode === DeleteMode.Delete) {
    const call = deleteDeleteApi.endpoint(routes.DeleteComponents);
    const { req } = await call.delete({
      componentIds: deleteComponentIds.value,
      forceErase: false,
    });
    if (deleteDeleteApi.ok(req)) {
      deleteModalRef.value?.close();
    }
  } else {
    const call = deleteEraseFromViewApi.endpoint(
      routes.EraseComponentsFromView,
      { viewId: explore.viewId.value },
    );
    const { req } = await call.delete({
      componentIds: deleteComponentIds.value,
    });
    if (deleteEraseFromViewApi.ok(req)) {
      deleteModalRef.value?.close();
    }
  }
};

const duplicateActionApi = useApi();
const componentDuplicate = async (componentIds: ComponentId[]) => {
  const call = duplicateActionApi.endpoint(routes.DuplicateComponents, {
    viewId: explore.viewId.value,
  });
  await call.post({
    components: componentIds,
  });
};

const upgradeActionApi = useApi();
const componentUpgrade = async (componentIds: ComponentId[]) => {
  const call = upgradeActionApi.endpoint(routes.UpgradeComponents);
  await call.post({
    componentIds,
  });
};

// eslint-disable-next-line @typescript-eslint/ban-types
const anchor = ref<Object | undefined>(undefined);

function open(
  // eslint-disable-next-line @typescript-eslint/ban-types
  anchorTo: Object,
  componentsForMenu: BifrostComponentInList[],
) {
  anchor.value = anchorTo;
  components.value = componentsForMenu;
  nextTick(() => contextMenuRef.value?.open());
}

function close() {
  components.value = [];
  contextMenuRef.value?.forceClose();
}

const isOpen = computed(() => contextMenuRef.value?.isOpen);

const emit = defineEmits<{
  (e: "edit"): void;
}>();

defineExpose({
  open,
  close,
  isOpen,
  componentsStartErase,
  componentDuplicate,
  componentUpgrade,
  contextMenuRef,
  componentsStartDelete,
  componentsRestore,
});
</script>
