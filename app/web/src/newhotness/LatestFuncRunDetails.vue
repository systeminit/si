<template>
  <FuncRunDetails v-if="latestFuncRunId" :funcRunId="latestFuncRunId" />
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
import { computed } from "vue";
import { Icon } from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import { ActionProposedView } from "@/store/actions.store";
import { useMakeKey, bifrost, useMakeArgs } from "@/store/realtime/heimdall";
import { BifrostActionViewList, EntityKind } from "@/workers/types/dbinterface";
import FuncRunDetails from "./FuncRunDetails.vue";
import { FunctionKind } from "./types";

const props = defineProps<{
  functionKind: FunctionKind;
  actionId?: string;
}>();

const key = useMakeKey();
const args = useMakeArgs();
const actionViewListRaw = useQuery<BifrostActionViewList | null>({
  queryKey: key(EntityKind.ActionViewList),
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(args(EntityKind.ActionViewList)),
});
const action = computed(
  () =>
    (actionViewListRaw.data.value?.actions.find(
      (a) => a.id === props.actionId,
    ) as ActionProposedView) ?? [],
);

export interface ActionProposedViewWithHydratedChildren
  extends ActionProposedView {
  dependentOnActions: ActionProposedView[];
  myDependentActions: ActionProposedView[];
  holdStatusInfluencedByActions: ActionProposedView[];
}

const latestFuncRunId = computed(() => {
  if (!action.value?.funcRunId) return null;
  return action.value.funcRunId;
});
</script>
