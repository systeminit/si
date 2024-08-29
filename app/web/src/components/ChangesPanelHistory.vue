<template>
  <ScrollArea
    v-if="actionsStore.historyActions.length > 0"
    ref="mainDivRef"
    @click="deselectOnClickEmptySpace"
  >
    <!-- TODO(Wendy)- SEARCH BAR SHOULD GO HERE -->
    <ActionsList
      v-for="[detail, actions] in actionsStore.historyActionsGrouped"
      :key="detail.changeSetId"
      :actions="actions"
      :changeSet="getChangeSet(detail)"
      :clickAction="clickAction"
      :selectedFuncRunIds="selectedFuncRunId ? [selectedFuncRunId] : []"
      kind="history"
      @history="openHistory"
    />
    <FuncRunTabGroup
      :close="deselectAction"
      :funcRun="funcRun"
      :selectedAction="selectedAction"
      :selectedTab="selectedTab"
    />
  </ScrollArea>
  <EmptyStateCard
    v-else
    iconName="actions"
    primaryText="No Actions Have Been Taken"
    secondaryText="There is no action history to display for this change set."
  />
</template>

<script lang="ts" setup>
import { ScrollArea } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import {
  ActionHistoryView,
  ActionProposedView,
  ChangeSetDetail,
  useActionsStore,
} from "@/store/actions.store";
import { FuncRun, FuncRunId, useFuncRunsStore } from "@/store/func_runs.store";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import EmptyStateCard from "./EmptyStateCard.vue";
import ActionsList from "./Actions/ActionsList.vue";
import FuncRunTabGroup from "./Actions/FuncRunTabGroup.vue";

const actionsStore = useActionsStore();
const funcRunsStore = useFuncRunsStore();

const mainDivRef = ref();
const selectedFuncRunId = ref<FuncRunId | undefined>();

const funcRun = ref<FuncRun | undefined>();

const selectedTab = ref<string | undefined>();

const selectedAction = computed(() => {
  if (selectedFuncRunId.value) {
    return actionsStore.historyActionsByFuncRunId.get(
      selectedFuncRunId.value,
    ) as ActionHistoryView;
  } else {
    return undefined;
  }
});

const getChangeSet = (detail: ChangeSetDetail) => {
  return {
    id: detail.changeSetId,
    name: detail.changeSetName,
    status: ChangeSetStatus.Applied,
    appliedAt: detail.timestamp && detail.timestamp.toISOString(),
    baseChangeSetId: "we dont need it",
  } as ChangeSet;
};

async function getFuncRun(funcRunId: FuncRunId | undefined) {
  if (funcRunId) {
    await funcRunsStore.GET_FUNC_RUN(funcRunId);
    funcRun.value = funcRunsStore.funcRuns[funcRunId];
  }
}

async function openHistory(id: FuncRunId, slug: string) {
  selectedFuncRunId.value = id;
  await getFuncRun(id);
  selectedTab.value = slug;
}

const deselectAction = () => {
  selectedFuncRunId.value = undefined;
};

const clickAction = async (
  action: ActionHistoryView | ActionProposedView,
): Promise<void> => {
  const a = action as ActionHistoryView;
  if (selectedFuncRunId.value === a.funcRunId) {
    deselectAction();
  } else {
    selectedFuncRunId.value = a.funcRunId;
    await getFuncRun(a.funcRunId);
  }
};

const deselectOnClickEmptySpace = (e: MouseEvent) => {
  const deselectArea = mainDivRef.value.$el.querySelector(".scroll-slot");
  if (e.target === deselectArea) {
    deselectAction();
  }
};
</script>
