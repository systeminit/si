<!-- eslint-disable vue/no-multiple-template-root -->
<template>
  <FuncRunDetailsLayout
    v-if="currentAction && currentAction.state === ActionState.Queued"
    :displayName="'Queued: ' + currentAction.name || 'Action Run'"
    noLogs
    :status="''"
    :logText="''"
    :errorHint="''"
    :errorMessageRaw="''"
  >
    <template #headerList>
      <template
        v-if="currentAction.kind && currentAction.kind !== currentAction.name"
      >
        <dt><Icon name="func" size="xs" /></dt>
        <dd>{{ currentAction.kind }}</dd>
      </template>

      <template v-if="currentAction.componentSchemaName">
        <dt></dt>
        <dd>{{ currentAction.componentSchemaName }}</dd>
      </template>

      <template v-if="currentAction.componentName">
        <dt></dt>
        <dd>{{ currentAction.componentName }}</dd>
      </template>
    </template>

    <template #grid>
      <GridItemWithLiveHeader class="row-span-3" title="Code" :live="false">
        <CodeViewer
          v-if="functionCode"
          :code="functionCode"
          language="javascript"
          allowCopy
        />
        <div v-else class="text-neutral-400 italic text-xs p-xs">
          No code available
        </div>
      </GridItemWithLiveHeader>

      <GridItemWithLiveHeader
        class="row-span-3"
        title="Arguments"
        :live="false"
      >
        <CodeViewer
          v-if="argsJson"
          :code="argsJson"
          language="json"
          allowCopy
        />
        <div v-else class="text-neutral-400 italic text-xs p-xs">
          No arguments available
        </div>
      </GridItemWithLiveHeader>
    </template>
  </FuncRunDetailsLayout>
  <FuncRunDetails v-else-if="funcRunId" :funcRunId="funcRunId" />
  <div
    v-else
    class="flex items-center justify-center h-full bg-neutral-900 text-white"
  >
    <div class="text-center p-6">
      <Icon name="loader" size="lg" class="mb-sm mx-auto text-action-500" />
      <p class="text-neutral-400">Looking for details...</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { useQuery, useQueryClient } from "@tanstack/vue-query";
import { computed, watch } from "vue";
import { bifrost, useMakeArgs, useMakeKey } from "@/store/realtime/heimdall";
import { ActionState } from "@/api/sdf/dal/action";
import {
  BifrostActionViewList,
  EntityKind,
} from "@/workers/types/entity_kind_types";
import CodeViewer from "@/components/CodeViewer.vue";
import FuncRunDetailsLayout from "./layout_components/FuncRunDetailsLayout.vue";
import GridItemWithLiveHeader from "./layout_components/GridItemWithLiveHeader.vue";
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

type QueuedDetails = { code: string; args: string };
const actionDetailQuery = useQuery<QueuedDetails>({
  queryKey: ["action_queued_details", props.actionId],
  staleTime: 100,
  enabled: () =>
    !!props.actionId && currentAction.value?.state === ActionState.Queued,
  queryFn: async () => {
    const call = api.endpoint<QueuedDetails>(routes.ActionQueuedDetails, {
      id: props.actionId as string,
    });
    const resp = await call.get();
    return resp.data;
  },
});

const functionCode = computed(() => actionDetailQuery.data.value?.code);

const argsJson = computed(() => {
  try {
    return JSON.stringify(actionDetailQuery.data.value?.args, null, 2);
  } catch (e) {
    return "// Error formatting arguments";
  }
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
