<template>
  <ScrollArea
    v-if="actionsStore.historyActions.length > 0"
    ref="mainDivRef"
    @click="deselectOnClickEmptySpace"
  >
    <!-- TODO(Wendy)- SEARCH BAR SHOULD GO HERE -->
    <ActionsList
      v-for="(actions, changeSetId) in actionsStore.historyActionsByChangeSetId"
      :key="changeSetId"
      :changeSet="getChangeSet(changeSetId)"
      :actions="actions"
      kind="history"
      :clickAction="clickAction"
      :selectedActionIds="selectedActionId ? [selectedActionId] : []"
      @history="openHistory"
    />
    <Teleport to=".si-panel-right">
      <Transition
        enterActiveClass="duration-100 ease-out"
        enterFromClass="translate-x-[500px]"
        leaveActiveClass="duration-100 ease-in"
        leaveToClass="translate-x-[500px]"
      >
        <div
          v-if="selectedAction"
          class="absolute w-[500px] h-full left-[-500px] bg-neutral-800 z-[-10]"
        >
          <TabGroup
            ref="historyDetailsTabGroupRef"
            variant="fullsize"
            @closeButtonTabClicked="deselectAction"
          >
            <TabGroupCloseButton />
            <ChangesPanelHistorySubpanelTab
              label="Resource Result"
              slug="resourceResult"
              emptyStateSecondaryTextNeedsAnA
              :data="selectedAction.resourceResult"
            />
            <ChangesPanelHistorySubpanelTab
              label="Code Executed"
              slug="codeExecuted"
              :data="selectedAction.codeExecuted"
            />
            <ChangesPanelHistorySubpanelTab
              label="Logs"
              slug="logs"
              :data="selectedAction.logs"
            />
            <ChangesPanelHistorySubpanelTab
              label="Arguments"
              slug="arguments"
              :data="selectedAction.arguments"
            />
          </TabGroup>
        </div>
      </Transition>
    </Teleport>
  </ScrollArea>
  <EmptyStateCard
    v-else
    iconName="actions"
    primaryText="No Actions Have Been Taken"
    secondaryText="There is no action history to display for this change set."
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  ScrollArea,
  TabGroup,
  TabGroupCloseButton,
} from "@si/vue-lib/design-system";
import { computed, nextTick, ref } from "vue";
import { ActionView, useActionsStore, ActionId } from "@/store/actions.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import {
  ChangeSet,
  ChangeSetId,
  ChangeSetStatus,
} from "@/api/sdf/dal/change_set";
import EmptyStateCard from "./EmptyStateCard.vue";
import ActionsList from "./Actions/ActionsList.vue";
import ChangesPanelHistorySubpanelTab from "./ChangesPanelHistorySubpanelTab.vue";

const actionsStore = useActionsStore();
const changeSetsStore = useChangeSetsStore();

const mainDivRef = ref();
const historyDetailsTabGroupRef = ref<InstanceType<typeof TabGroup>>();
const selectedActionId = ref<ActionId | undefined>();

const selectedAction = computed(() => {
  if (selectedActionId.value) {
    const action = actionsStore.historyActionsById.get(selectedActionId.value);
    return action;
  } else return undefined;
});

const getChangeSet = (changeSetId: ChangeSetId) => {
  const changeSet = changeSetsStore.changeSetsById[changeSetId];

  if (changeSet) return changeSet;
  else {
    // TODO(Wendy) - for now if we get an invalid changeSetId we will put mock data in, not sure what to do instead?
    return {
      id: changeSetId,
      name: "mock changeset",
      status: ChangeSetStatus.Applied,
      appliedAt: new Date().toISOString(),
      baseChangeSetId: "mock data not a real changeSetId",
    } as ChangeSet;
  }
};

function openHistory(id: ActionId, slug: string) {
  selectedActionId.value = id;
  nextTick(() => {
    historyDetailsTabGroupRef.value?.selectTab(slug);
  });
}

const deselectAction = () => {
  selectedActionId.value = undefined;
};

const clickAction = (action: ActionView) => {
  if (selectedActionId.value === action.id) {
    deselectAction();
  } else {
    selectedActionId.value = action.id;
  }
};

const deselectOnClickEmptySpace = (e: MouseEvent) => {
  const deselectArea = mainDivRef.value.$el.querySelector(".scroll-slot");
  if (e.target === deselectArea) {
    deselectAction();
  }
};
</script>
