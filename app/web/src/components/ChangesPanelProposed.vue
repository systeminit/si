<template>
  <div v-if="actionsStore.proposedActions.length > 0">
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
    <ConfirmHoldModal ref="confirmRef" :ok="finishHold" />
    <ActionsList
      :clickAction="clickAction"
      :selectedActionIds="selectedActionIds"
      kind="proposed"
    />

    <FuncRunTabGroup
      :close="deselectAction"
      :funcRun="funcRun"
      :selectedAction="singleSelectedAction"
      :selectedTab="selectedTab"
    />
  </div>
  <EmptyStateCard
    v-else
    iconName="actions"
    primaryText="No Actions Have Been Proposed"
    secondaryText="Propose some actions in this change set to see them here."
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ref, reactive, computed, watch, WatchStopHandle } from "vue";
import { VButton } from "@si/vue-lib/design-system";
import { ActionId, ActionState } from "@/api/sdf/dal/action";
import {
  useActionsStore,
  ActionProposedView,
  ActionView,
} from "@/store/actions.store";
import FuncRunTabGroup from "@/components/Actions/FuncRunTabGroup.vue";
import { FuncRun, useFuncRunsStore } from "@/store/func_runs.store";
import ConfirmHoldModal from "./Actions/ConfirmHoldModal.vue";
import ActionsList from "./Actions/ActionsList.vue";
import EmptyStateCard from "./EmptyStateCard.vue";

const actionsStore = useActionsStore();
const funcRunsStore = useFuncRunsStore();

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

const clickAction = async (action_view: ActionView, e: MouseEvent) => {
  const action = action_view as ActionProposedView;

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
        const updatedAction = actionsStore.actionsById.get(action.id);
        if (updatedAction && updatedAction.funcRunId) {
          await funcRunsStore.GET_FUNC_RUN(updatedAction.funcRunId);
          funcRun.value = funcRunsStore.funcRuns[updatedAction.funcRunId];
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
