<template>
  <ScrollArea ref="mainDivRef" @click="deselectOnClickEmptySpace">
    <TreeNode
      enableDefaultHoverClasses
      enableGroupToggle
      alwaysShowArrow
      indentationSize="none"
      leftBorderSize="none"
      defaultOpen
      internalScrolling
      class="min-h-[32px]"
      primaryIconClasses=""
      label="Actions"
    >
      <!-- TODO(Wendy)- SEARCH BAR SHOULD GO HERE -->
      <template v-if="actionsStore.historyActions.length > 0">
        <ActionsList
          v-for="[detail, actions] in actionsStore.historyActionsGrouped"
          :key="detail.changeSetId"
          :actions="actions"
          :changeSet="getChangeSet(detail)"
          :clickAction="clickActionOrMgmtRun"
          :selectedFuncRunIds="selectedFuncRunId ? [selectedFuncRunId] : []"
          kind="history"
          @history="openHistory"
        />
      </template>
      <EmptyStateCard
        v-else
        iconName="actions"
        primaryText="No Actions Have Been Executed"
        secondaryText="There is no action history to display for this change set."
      />
    </TreeNode>
    <TreeNode
      enableDefaultHoverClasses
      enableGroupToggle
      alwaysShowArrow
      indentationSize="none"
      leftBorderSize="none"
      defaultOpen
      internalScrolling
      class="min-h-[32px]"
      primaryIconClasses=""
      label="Management Functions"
    >
      <ManagementHistoryList
        v-if="managementHistoryForChangeSet.length > 0"
        :managementHistory="managementHistoryForChangeSet"
        :clickItem="clickActionOrMgmtRun"
        :funcRunId="selectedFuncRunId"
        @history="openHistory"
      />
      <EmptyStateCard
        v-else
        iconName="actions"
        primaryText="No Management Functions Have Been Executed"
        secondaryText="There is no management function history to display for this change set."
      />
    </TreeNode>

    <FuncRunTabGroup
      :close="deselectActionOrMgmtRun"
      :funcRun="funcRun"
      :open="!!selectedFuncRunId"
      :selectedTab="selectedTab"
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import { ScrollArea, TreeNode } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import {
  ActionHistoryView,
  ActionProposedView,
  ChangeSetDetail,
  useActionsStore,
} from "@/store/actions.store";
import {
  FuncRun,
  FuncRunId,
  ManagementHistoryItem,
  useFuncRunsStore,
} from "@/store/func_runs.store";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useChangeSetsStore } from "@/store/change_sets.store";
import EmptyStateCard from "./EmptyStateCard.vue";
import ActionsList from "./Actions/ActionsList.vue";
import FuncRunTabGroup from "./Actions/FuncRunTabGroup.vue";
import ManagementHistoryList from "./Management/ManagementHistoryList.vue";

const actionsStore = useActionsStore();
const funcRunsStore = useFuncRunsStore();
const changeSetsStore = useChangeSetsStore();

const mainDivRef = ref();
const selectedFuncRunId = ref<FuncRunId | undefined>();

const funcRun = ref<FuncRun | undefined>();

const selectedTab = ref<string | undefined>();

const managementHistoryForChangeSet = computed(() =>
  changeSetsStore.selectedChangeSetId
    ? funcRunsStore.managementRunHistory[changeSetsStore.selectedChangeSetId] ??
      []
    : [],
);

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

const deselectActionOrMgmtRun = () => {
  selectedFuncRunId.value = undefined;
};

const clickActionOrMgmtRun = async (
  run: ActionHistoryView | ActionProposedView | ManagementHistoryItem,
): Promise<void> => {
  if (selectedFuncRunId.value === run.funcRunId) {
    deselectActionOrMgmtRun();
  } else {
    selectedFuncRunId.value = run.funcRunId;
    await getFuncRun(run.funcRunId);
  }
};

const deselectOnClickEmptySpace = (e: MouseEvent) => {
  const deselectArea = mainDivRef.value.$el.querySelector(".scroll-slot");
  if (e.target === deselectArea) {
    deselectActionOrMgmtRun();
  }
};
</script>
