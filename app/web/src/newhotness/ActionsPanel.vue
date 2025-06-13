<template>
  <EmptyState
    v-if="actionPrototypeViews.length === 0"
    text="No actions available"
    icon="tools"
  />
  <div v-else class="flex flex-col">
    <div
      class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
    >
      The changes below will run when you click "Apply Changes".
    </div>
    <ActionWidget
      v-for="actionPrototypeView in actionPrototypeViews"
      :key="actionPrototypeView.id"
      :actionPrototypeView="actionPrototypeView"
      :actionId="actionByPrototype[actionPrototypeView.id]"
      :component="props.component"
    />
  </div>
</template>

<script lang="ts" setup>
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  ActionPrototypeViewList,
  BifrostActionViewList,
  BifrostComponent,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import { ActionId, ActionPrototypeId } from "@/api/sdf/dal/action";
import ActionWidget from "./ActionWidget.vue";
import EmptyState from "./EmptyState.vue";

const props = defineProps<{
  component: BifrostComponent;
}>();

// The code below is the same as in AssetActionsDetails in the mead hall

// This is the core materialized view for this component. We need it to know what action prototypes
// are available for the given component.
const makeKey = useMakeKey();
const makeArgs = useMakeArgs();

const queryKeyForActionPrototypeViews = makeKey(
  EntityKind.ActionPrototypeViewList,
  props.component.schemaVariant.id,
);
const actionPrototypeViewsRaw = useQuery<ActionPrototypeViewList | null>({
  queryKey: queryKeyForActionPrototypeViews,
  queryFn: async () =>
    await bifrost<ActionPrototypeViewList>(
      makeArgs(
        EntityKind.ActionPrototypeViewList,
        props.component.schemaVariant.id,
      ),
    ),
});
const actionPrototypeViews = computed(
  () => actionPrototypeViewsRaw.data.value?.actionPrototypes ?? [],
);

// Use the materialized view for actions to know what actions exist for a given prototype and the
// selected component.
const queryKeyForActionViewList = makeKey(EntityKind.ActionViewList);
const actionViewList = useQuery<BifrostActionViewList | null>({
  queryKey: queryKeyForActionViewList,
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(makeArgs(EntityKind.ActionViewList)),
});
const actionByPrototype = computed(() => {
  if (!actionViewList.data.value?.actions) return {};
  if (actionViewList.data.value.actions.length < 1) return {};

  const result: Record<ActionPrototypeId, ActionId> = {};
  for (const action of actionViewList.data.value.actions) {
    if (action.componentId === props.component.id) {
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
</script>
