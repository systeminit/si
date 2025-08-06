<template>
  <FuncRunDetails v-if="funcRunId" :funcRunId="funcRunId" />
  <div
    v-else
    class="flex items-center justify-center h-full bg-neutral-900 text-white"
  >
    <div class="text-center p-6">
      <Icon name="loader" size="lg" class="mb-sm mx-auto text-action-500" />
      <p class="text-neutral-400">Looking for function run details...</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, watch } from "vue";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import {
  BifrostActionViewList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import FuncRunDetails from "./FuncRunDetails.vue";
import { ActionProposedView, FunctionKind } from "./types";
import { useApi, routes } from "./api_composables";

const api = useApi();
const queryClient = useQueryClient();

const key = useMakeKey();
const args = useMakeArgs();

const props = defineProps<{
  functionKind: FunctionKind;
  actionId: string;
}>();

// Query the action view list to watch for changes to our specific action
const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});

// Computed that finds our specific action
const currentAction = computed(() => {
  if (!props.actionId || !actionViewListRaw.data.value?.actions) return null;
  return actionViewListRaw.data.value.actions.find(
    (action) => action.id === props.actionId,
  );
});

const actionFuncRunQuery = useQuery<string>({
  queryKey: ["action_func_run_id", props.actionId],
  staleTime: 100,
  enabled: () => !!props.actionId,
  queryFn: async () => {
    const call = api.endpoint<{ funcRunId: string }>(routes.ActionFuncRunId, {
      id: props.actionId as string,
    });
    const resp = await call.get();
    return resp.data.funcRunId;
  },
});

const funcRunId = computed(() => actionFuncRunQuery.data.value);

// Cache busting: when the action itself changes, invalidate the query
watch(
  () => currentAction.value?.state,
  () => {
    if (props.actionId) {
      queryClient.invalidateQueries({
        queryKey: ["action_func_run_id", props.actionId],
      });
    }
  },
  { deep: true },
);

export interface ActionProposedViewWithHydratedChildren
  extends ActionProposedView {
  dependentOnActions: ActionProposedView[];
  myDependentActions: ActionProposedView[];
  holdStatusInfluencedByActions: ActionProposedView[];
}
</script>
