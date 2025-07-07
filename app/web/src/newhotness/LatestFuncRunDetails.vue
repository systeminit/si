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
import { useQuery } from "@tanstack/vue-query";
import { computed } from "vue";
import { ActionProposedView } from "@/store/actions.store";
import FuncRunDetails from "./FuncRunDetails.vue";
import { FunctionKind } from "./types";
import { useApi, routes } from "./api_composables";

const api = useApi();
const props = defineProps<{
  functionKind: FunctionKind;
  actionId?: string;
}>();

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

export interface ActionProposedViewWithHydratedChildren
  extends ActionProposedView {
  dependentOnActions: ActionProposedView[];
  myDependentActions: ActionProposedView[];
  holdStatusInfluencedByActions: ActionProposedView[];
}
</script>
