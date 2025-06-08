<template>
  <div>
    <DropdownMenu
      ref="contextMenuRef"
      :anchorTo="anchor"
      :items="rightClickMenuItems"
      variant="editor"
      noDefaultClose
      :alignOutsideRightEdge="onGrid"
      :alignOutsideLeftEdge="!onGrid"
      :overlapAnchorOffset="Y_OFFSET"
    />
    <EraseModal ref="eraseModalRef" @confirm="componentsFinishErase" />
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
  BifrostComponent,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { ActionId, ActionPrototypeId } from "@/api/sdf/dal/action";
import EraseModal from "./EraseModal.vue";
import { useApi, routes } from "./api_composables";
import { assertIsDefined, ExploreContext } from "./types";

const props = defineProps<{
  componentIds: string[];
  onGrid?: boolean;
}>();

// This number fixes the Y position to align with the ComponentGridTile
const Y_OFFSET = 4;

const contextMenuRef = ref<InstanceType<typeof DropdownMenu>>();

const key = useMakeKey();
const args = useMakeArgs();

const explore = inject<ExploreContext>("EXPLORE_CONTEXT");
assertIsDefined<ExploreContext>(explore);

// ================================================================================================
// This is the location of objects needed to populate menu items.
const ids = ref<ComponentId[]>([]);
// eslint-disable-next-line @typescript-eslint/no-non-null-assertion
const id = computed(() => (ids.value.length === 1 ? ids.value[0]! : ""));
const component = computed(() => componentQuery.data.value);
const schemaVariantId = computed(() => component.value?.schemaVariant.id ?? "");
// ================================================================================================

const componentQuery = useQuery<BifrostComponent | null>({
  enabled: id.value !== "",
  queryKey: key(EntityKind.Component, id),
  queryFn: async () =>
    await bifrost<BifrostComponent>(args(EntityKind.Component, id.value)),
});

const actionPrototypes = computed(
  () => actionPrototypesQuery.data.value?.actionPrototypes ?? [],
);
const actionPrototypesQuery = useQuery<ActionPrototypeViewList | null>({
  enabled: schemaVariantId.value !== "",
  queryKey: key(EntityKind.ActionPrototypeViewList, schemaVariantId),
  queryFn: async () =>
    await bifrost<ActionPrototypeViewList>(
      args(
        EntityKind.ActionPrototypeViewList,
        component.value?.schemaVariant.id,
      ),
    ),
});

const actionsQuery = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});

const actionByPrototype = computed(() => {
  if (!id.value) return {};
  if (!actionsQuery.data.value?.actions) return {};
  if (actionsQuery.data.value.actions.length < 1) return {};

  const result: Record<ActionPrototypeId, ActionId> = {};
  for (const action of actionsQuery.data.value.actions) {
    if (action.componentId === id.value) {
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

const addActionApi = useApi();
const removeActionApi = useApi();

const rightClickMenuItems = computed(() => {
  const items: DropdownMenuItemObjectDef[] = [];

  // can erase so long as you have not selected a view
  items.push({
    label: "Erase",
    shortcut: "⌘E",
    icon: "erase",
    onSelect: () => componentsStartErase(props.componentIds),
  });

  items.push({
    label: "Duplicate",
    shortcut: "⌘D",
    icon: "clipboard-copy",
    onSelect: () => componentDuplicate(props.componentIds),
  });

  if (component.value?.canBeUpgraded) {
    items.push({
      label: "Upgrade",
      // TODO(Wendy) - we need to implement this shortcut
      // shortcut: "⌘U",
      icon: "bolt-outline",
      onSelect: () => componentUpgrade(props.componentIds),
    });
  }

  // Only enable actions if we are working with a single component.
  if (id.value && schemaVariantId.value) {
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
              componentId: id.value,
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
        label: "Actions",
        submenuItems,
      });
    }
  }

  return items;
});

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
  componentIds: ComponentId[],
) {
  anchor.value = anchorTo;
  ids.value = componentIds;
  nextTick(() => contextMenuRef.value?.open());
}

function close() {
  ids.value = [];
  contextMenuRef.value?.close();
}

const isOpen = computed(() => contextMenuRef.value?.isOpen);

defineExpose({ open, close, isOpen, componentsStartErase, componentDuplicate });
</script>
