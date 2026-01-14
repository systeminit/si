<template>
  <TreeNode :enableGroupToggle="populated > 0" styleAsGutter>
    <template #label>
      <div
        :class="
          clsx(
            'flex flex-row gap-2xs justify-between place-items-center bg-neutral-100 dark:bg-neutral-700 text-sm',
            changeSet ? 'py-2xs px-xs' : 'py-xs',
          )
        "
      >
        <template v-if="changeSet">
          <div class="flex flex-col flex-grow min-w-0">
            <Timestamp
              :date="changeSet.appliedAt"
              :timeClasses="themeClasses('text-neutral-500', 'text-neutral-400')"
              class="text-sm"
              dateClasses="font-bold"
              showTimeIfToday
              size="long"
            />
            <div :class="clsx('text-xs truncate', themeClasses('text-neutral-500', 'text-neutral-400'))">
              <span class="font-bold">Change Set:</span> {{ changeSet.name }}
            </div>
          </div>
          <div class="flex-none font-bold">{{ populated }} Action(s)</div>
          <!-- TODO(Wendy) - maybe a PillCounter makes more sense here? -->
          <!-- <PillCounter
            v-tooltip="{ content: 'Actions', placement: 'left' }"
            :count="displayActions.length"
            class="flex-none font-bold cursor-pointer"
            size="lg"
          /> -->
        </template>
        <template v-else>
          <div class="grow-0 mx-[.66em]">
            <Icon class="attributes-panel-item__type-icon" name="bullet-list" size="sm" />
          </div>
          <div class="grow">{{ populated }} Action(s)</div>
          <div class="grow-0 flex flex-row mr-xs">
            <div
              v-for="(cnt, actionKind) in actionsByKind"
              :key="actionKind"
              class="flex flex-row mx-2xs p-2xs rounded dark:bg-neutral-900 bg-neutral-200"
            >
              <div class="mx-2xs">{{ cnt }}</div>
              <StatusIndicatorIcon :status="actionKind.toString()" size="sm" type="action" />
            </div>
          </div>
        </template>
      </div>
    </template>
    <template v-if="proposedActions">
      <div
        v-for="action in proposedActions"
        :key="action.id"
        :class="clsx('border-b', themeClasses('border-neutral-100', 'border-neutral-700'))"
      >
        <ActionCard
          :action="action"
          :noInteraction="props.noInteraction"
          :selected="isSelected(action)"
          :slim="props.slim"
          @click="props.clickAction && props.clickAction(action, $event)"
          @history="openHistory"
          @remove="actionsStore.CANCEL([action.id])"
        />
      </div>
    </template>
    <template v-if="historyActions">
      <div
        v-for="action in historyActions"
        :key="action.id"
        :class="clsx('border-b', themeClasses('border-neutral-100', 'border-neutral-700'))"
      >
        <ActionHistoryCard
          :action="action"
          :noInteraction="props.noInteraction"
          :selected="isSelected(action)"
          :slim="props.slim"
          @history="openHistory"
          @remove="actionsStore.CANCEL([action.id])"
        />
      </div>
    </template>
  </TreeNode>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { Icon, Timestamp, TreeNode, themeClasses } from "@si/vue-lib/design-system";
import { PropType, computed } from "vue";
import { useActionsStore, ActionView, ActionHistoryView, FuncRunId, ActionProposedView } from "@/store/actions.store";
import { DefaultMap } from "@/utils/defaultmap";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import ActionCard from "./ActionCard.vue";
import ActionHistoryCard from "./ActionHistoryCard.vue";

export interface ActionProposedViewWithHydratedChildren extends ActionProposedView {
  dependentOnActions: ActionProposedView[];
  myDependentActions: ActionProposedView[];
  holdStatusInfluencedByActions: ActionProposedView[];
}

export type ActionsListKind = "proposed" | "history";

const actionsStore = useActionsStore();

type clickFn = (action: ActionProposedViewWithHydratedChildren, e: MouseEvent) => void;

const props = defineProps({
  proposedActions: {
    type: Array<ActionProposedViewWithHydratedChildren>,
    required: false,
  },
  historyActions: { type: Array<ActionHistoryView>, required: false },
  noInteraction: { type: Boolean },
  selectedActionIds: { type: Array<string> },
  selectedFuncRunIds: { type: Array<string> },
  slim: { type: Boolean },
  clickAction: {
    type: Function as PropType<clickFn>,
    default: undefined,
  },
  changeSet: { type: Object as PropType<ChangeSet> },
});

const populated = computed(() => (props.proposedActions?.length ?? 0) || (props.historyActions?.length ?? 0));

const selectedIds = computed<string[]>(() => {
  if (props.proposedActions) {
    return props.selectedActionIds || [];
  }
  return props.selectedFuncRunIds || [];
});

const isSelected = (action: ActionView) => {
  let id: string | undefined;
  if (props.historyActions) {
    id = action.funcRunId;
  } else {
    id = action.id;
  }
  if (id) return selectedIds.value.includes(id);
  return false;
};

const actionsByKind = computed(() => {
  const actions = props.proposedActions ?? props.historyActions;
  if (!actions) return {};
  const counts = new DefaultMap<string, number>(() => 0);
  for (const action of actions) {
    counts.set(action.kind, counts.get(action.kind) + 1);
  }
  return Object.fromEntries(counts);
});

const emit = defineEmits<{
  (e: "history", id: FuncRunId, tabSlug: string): void;
}>();

function openHistory(id: FuncRunId, slug: string) {
  emit("history", id, slug);
}
</script>
