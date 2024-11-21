<template>
  <div
    v-if="featureFlagsStore.MANAGEMENT_FUNCTIONS"
    class="w-full h-full flex flex-col overflow-hidden relative"
    @click="deselectOnClickEmptySpace"
  >
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
      <div ref="actionDivRef" class="py-2xs w-full h-full">
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
      </div>
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
      <div
        ref="managementDivRef"
        :class="
          clsx(
            'w-full h-full',
            managementHistoryForChangeSet.length === 0 && 'py-2xs',
          )
        "
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
      </div>
    </TreeNode>

    <FuncRunTabGroup
      :close="deselectActionOrMgmtRun"
      :funcRun="funcRun"
      :open="!!selectedFuncRunId"
      :selectedTab="selectedTab"
    />
  </div>
  <div v-else ref="actionDivRef" class="py-2xs w-full h-full">
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
  </div>
</template>

<script lang="ts" setup>
import { TreeNode } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import clsx from "clsx";
import {
  ActionHistoryView,
  ActionProposedView,
  ChangeSetDetail,
  useActionsStore,
} from "@/store/actions.store";
import { FuncRun, FuncRunId, useFuncRunsStore } from "@/store/func_runs.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { useChangeSetsStore } from "@/store/change_sets.store";
import {
  ManagementHistoryItem,
  useManagementRunsStore,
} from "@/store/management_runs.store";
import EmptyStateCard from "./EmptyStateCard.vue";
import ActionsList from "./Actions/ActionsList.vue";
import FuncRunTabGroup from "./Actions/FuncRunTabGroup.vue";
import ManagementHistoryList from "./Management/ManagementHistoryList.vue";

const actionsStore = useActionsStore();
const funcRunsStore = useFuncRunsStore();
const managementRunsStore = useManagementRunsStore();
const changeSetsStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();

const actionDivRef = ref();
const managementDivRef = ref();
const selectedFuncRunId = ref<FuncRunId | undefined>();

const funcRun = ref<FuncRun | undefined>();

const selectedTab = ref<string | undefined>();

const managementHistoryForChangeSet = computed(() =>
  changeSetsStore.selectedChangeSetId
    ? managementRunsStore.managementRunHistory ?? []
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
  const deselectArea1 = actionDivRef.value;
  const deselectArea2 = managementDivRef.value;
  if (e.target === deselectArea1 || e.target === deselectArea2) {
    deselectActionOrMgmtRun();
  }
};
</script>
