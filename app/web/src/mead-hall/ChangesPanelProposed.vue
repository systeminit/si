<template>
  <div class="h-full flex flex-col overflow-hidden">
    <ConfirmHoldModal ref="confirmRef" :ok="finishHold" />
    <div v-if="proposedActions.length > 0">
      <!-- TODO(Wendy)- SEARCH BAR SHOULD GO HERE -->
      <div class="flex flex-row place-content-center">
        <VButton
          :disabled="disabledMultiple"
          class="flex-1 m-xs dark:hover:bg-action-900 hover:bg-action-100 dark:hover:text-action-300 hover:text-action-700 hover:underline"
          icon="circle-stop"
          iconClass="text-warning-400"
          label="Put On Hold"
          size="xs"
          tone="empty"
          variant="solid"
          @click="holdAll"
        />
        <VButton
          :disabled="disabledMultiple"
          class="flex-1 m-xs dark:hover:bg-action-900 hover:bg-action-100 dark:hover:text-action-300 hover:text-action-700 hover:underline"
          icon="x"
          iconClass="dark:text-destructive-600 text-destructive-500"
          label="Remove"
          size="xs"
          tone="empty"
          variant="solid"
          @click="removeAll"
        />
      </div>
    </div>
    <FuncRunTabGroup
      :close="deselectAction"
      :funcRun="funcRun"
      :open="!!singleSelectedAction"
      :selectedTab="selectedTab"
    />
    <template v-if="changeSetStore.headSelected">
      <ActionsList
        v-if="allActionViews.length > 0"
        :clickAction="clickAction"
        :selectedActionIds="selectedActionIds"
        :proposedActions="allActionViews"
      />
      <EmptyStateCard
        v-else
        iconName="actions"
        primaryText="All Actions on HEAD have been run"
        secondaryText="You can see those actions in the history tab."
      />
    </template>
    <template v-else>
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
        label="Proposed Actions In This Change Set"
      >
        <ActionsList
          v-if="proposedActions.length > 0"
          class="mt-sm"
          :clickAction="clickAction"
          :selectedActionIds="selectedActionIds"
          :proposedActions="proposedActions"
        />
        <EmptyStateCard
          v-else
          iconName="actions"
          primaryText="No Actions Have Been Proposed In This Change Set"
          secondaryText="Propose some actions to see them here."
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
        label="Actions In Queue on HEAD"
      >
        <ActionsList
          v-if="headActions.length > 0"
          class="mt-sm"
          :selectedActionIds="selectedActionIds"
          :proposedActions="headActions"
          noInteraction
        />
        <EmptyStateCard
          v-else
          iconName="actions"
          primaryText="All Actions on HEAD have run"
          secondaryText="See past actions in the history tab."
        />
      </TreeNode>
    </template>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ref, reactive, computed, watch, WatchStopHandle } from "vue";
import { VButton, TreeNode } from "@si/vue-lib/design-system";
import { useQuery } from "@tanstack/vue-query";
import { ActionId, ActionState } from "@/api/sdf/dal/action";
import FuncRunTabGroup from "@/components/Actions/FuncRunTabGroup.vue";
import { FuncRun, useFuncRunsStore } from "@/store/func_runs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ConfirmHoldModal from "@/components/Actions/ConfirmHoldModal.vue";
import EmptyStateCard from "@/components/EmptyStateCard.vue";
import { bifrost, makeArgs, makeKey } from "@/store/realtime/heimdall";
import { ActionProposedView, useActionsStore } from "@/store/actions.store";
import { BifrostActionViewList } from "@/workers/types/entity_kind_types";
import ActionsList from "./ActionsList.vue";

export interface ActionProposedViewWithHydratedChildren
  extends ActionProposedView {
  dependentOnActions: ActionProposedView[];
  myDependentActions: ActionProposedView[];
  holdStatusInfluencedByActions: ActionProposedView[];
}

const actionsStore = useActionsStore();
const funcRunsStore = useFuncRunsStore();
const changeSetStore = useChangeSetsStore();

const queryKey = makeKey("ActionViewList");
const actionViewList = useQuery<BifrostActionViewList | null>({
  queryKey,
  queryFn: async () =>
    await bifrost<BifrostActionViewList>(makeArgs("ActionViewList")),
});

// Materialized views cannot, yet, build circular references
// Ideally, this gets moved lower in the stack, with generated code
const allActionViews = computed(() => {
  if (!actionViewList.data.value) return [];
  if (actionViewList.data.value.actions.length < 1) return [];
  const proposed = actionViewList.data.value.actions;
  const proposedById = proposed.reduce((obj, p) => {
    obj[p.id] = p;
    return obj;
  }, {} as Record<string, ActionProposedView>);
  return proposed.map((_p) => {
    const p = { ..._p } as ActionProposedViewWithHydratedChildren;
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    p.dependentOnActions = p.dependentOn.map((d) => proposedById[d]!);
    // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
    p.myDependentActions = p.myDependencies.map((d) => proposedById[d]!);
    p.holdStatusInfluencedByActions = p.holdStatusInfluencedBy.map(
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      (d) => proposedById[d]!,
    );
    return p;
  });
});
const proposedActions = computed(() =>
  allActionViews.value.filter(
    // NOTE(nick): this was ported over, but this is really not the right way to check HEAD.
    (apv) => apv.originatingChangeSetId === changeSetStore.selectedChangeSetId,
  ),
);
const headActions = computed(() =>
  allActionViews.value.filter(
    // NOTE(nick): this was ported over, but this is really not the right way to check HEAD.
    (apv) => apv.originatingChangeSetId !== changeSetStore.selectedChangeSetId,
  ),
);

const confirmRef = ref<InstanceType<typeof ConfirmHoldModal> | null>(null);

const selectedActions: Map<ActionId, ActionProposedView> = reactive(new Map());

const singleSelectedAction = computed(() =>
  selectedActions.size === 1
    ? selectedActions.values().next().value
    : undefined,
);

const selectedActionIds = computed(() =>
  Object.keys(Object.fromEntries(selectedActions)),
);

const disabledMultiple = computed(() => selectedActions.size === 0);

const holdAll = () => {
  const actions = Object.values(Object.fromEntries(selectedActions));
  if (_.some(actions, (a) => a.myDependencies.length > 0))
    confirmRef.value?.open();
  else finishHold();
};

const finishHold = (): void => {
  if (selectedActionIds.value.length > 0)
    actionsStore.PUT_ACTION_ON_HOLD(selectedActionIds.value);
  confirmRef.value?.close();
};

const removeAll = () => {
  if (selectedActionIds.value.length > 0)
    actionsStore.CANCEL(selectedActionIds.value);
};

const funcRun = ref<FuncRun | undefined>();
const selectedTab = ref<string | undefined>();

let funcRunWatcher: WatchStopHandle | undefined;

const clickAction = async (
  action: ActionProposedViewWithHydratedChildren,
  e: MouseEvent,
) => {
  if (e.shiftKey) {
    if (!selectedActions.has(action.id)) {
      selectedActions.set(action.id, action as ActionProposedView);
    } else selectedActions.delete(action.id);
  } else {
    const singleSelectionActionId = singleSelectedAction.value?.id;
    selectedActions.clear();

    if (singleSelectionActionId === action.id) {
      funcRun.value = undefined;
      return;
    }

    selectedActions.set(action.id, action as ActionProposedView);

    const { funcRunId } = action;

    if (!funcRunId) {
      funcRun.value = undefined;
      return;
    }

    if (funcRunWatcher) {
      funcRunWatcher();
    }
    funcRunWatcher = watch(
      () => funcRunsStore.lastRuns[action.id],
      async () => {
        // we don't want the variable from the closure b/c
        // the actions list has been updated behind the scenes
        // and it has a new fun run id, go get it and load that func run
        if (action && action.funcRunId) {
          await funcRunsStore.GET_FUNC_RUN(action.funcRunId);
          funcRun.value = funcRunsStore.funcRuns[action.funcRunId];
        }
      },
    );

    await funcRunsStore.GET_FUNC_RUN(funcRunId);
    funcRun.value = funcRunsStore.funcRuns[funcRunId];

    if ([ActionState.Queued, ActionState.OnHold].includes(action.state)) {
      selectedTab.value = "arguments";
    } else {
      selectedTab.value = "logs";
    }
  }
};

const deselectAction = () => {
  selectedActions.clear();
};

defineProps({
  old: { type: Boolean },
});
</script>
