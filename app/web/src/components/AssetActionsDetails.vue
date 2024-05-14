<template>
  <div class="h-full relative">
    <TabGroup
      variant="minimal"
      :startSelectedTabSlug="componentsStore.detailsTabSlugs[1] || undefined"
      marginTop="2xs"
      @update:selectedTab="onTabSelected"
    >
      <TabGroupItem label="Select" slug="actions-selection">
        <template v-if="actionsReqStatus.isPending">
          Loading actions...</template
        >
        <template v-else-if="actionsReqStatus.isError">
          <ErrorMessage :requestStatus="actionsReqStatus" />
        </template>
        <template
          v-else-if="actionsReqStatus.isSuccess && selectedComponentActions"
        >
          <div
            v-if="selectedComponentActions.length === 0"
            class="flex flex-col items-center pt-lg h-full w-full text-neutral-400"
          >
            <div class="w-64">
              <EmptyStateIcon name="no-changes" />
            </div>
            <span class="text-xl">No Actions available</span>
          </div>
          <div v-else class="flex flex-col">
            <div
              class="text-sm text-neutral-700 dark:text-neutral-300 p-xs italic border-b dark:border-neutral-600"
            >
              Select the actions you want to run below. Actions will be enacted
              after this change set has been applied. To do so, deselect this
              component and click the "Apply Changes" button in the top right.
            </div>
            <ActionWidget
              v-for="action in selectedComponentActions"
              :key="action.actionPrototypeId"
              :componentId="componentId"
              :actionPrototypeId="action.actionPrototypeId"
            />
          </div>
        </template>
      </TabGroupItem>
      <TabGroupItem slug="actions-history">
        <template #label>
          <Inline>
            <span>History</span>
            <!-- <PillCounter class="ml-2xs" :count="filteredBatches.length" /> -->
          </Inline>
        </template>

        <div
          v-if="filteredBatches.length === 0"
          class="flex flex-col items-center pt-lg h-full w-full text-neutral-400"
        >
          <div class="w-64">
            <EmptyStateIcon name="no-changes" />
          </div>
          <span class="text-xl">No actions history</span>
        </div>
        <ul v-else class="flex flex-col">
          <li v-for="(actionBatch, index) in filteredBatches" :key="index">
            <ApplyHistoryItem :actionBatch="actionBatch" :collapse="false" />
          </li>
        </ul>
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType, ref, watch } from "vue";
import * as _ from "lodash-es";

import {
  ErrorMessage,
  Inline,
  TabGroup,
  TabGroupItem,
} from "@si/vue-lib/design-system";

import { useComponentsStore } from "@/store/components.store";
import { ComponentId } from "@/api/sdf/dal/component";

import ApplyHistoryItem from "@/components/ApplyHistoryItem.vue";
import { useActionsStore } from "@/store/actions.store";
import EmptyStateIcon from "@/components/EmptyStateIcon.vue";
import ActionWidget from "@/components/ActionWidget.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const actionsStore = useActionsStore();
const componentsStore = useComponentsStore();
const changeSetsStore = useChangeSetsStore();

const actionBatches = computed(() =>
  _.reverse([...actionsStore.actionBatches]),
);

const filteredBatches = computed(() =>
  actionBatches.value
    .map((batch) => ({
      ...batch,
      actions: batch.actions.filter(
        (action) => action.componentId === props.componentId,
      ),
    }))
    .filter((batch) => batch.actions.length),
);

const tabsRef = ref<InstanceType<typeof TabGroup>>();
function onTabSelected(newTabSlug?: string) {
  componentsStore.setComponentDetailsTab(newTabSlug || null);
}

const actionsReqStatus = actionsStore.getRequestStatus(
  "FETCH_COMPONENT_ACTIONS",
  props.componentId,
);

const selectedComponentActions = computed(
  () => actionsStore.actionsByComponentId[props.componentId],
);

watch(
  () => componentsStore.selectedComponentDetailsTab,
  (tabSlug) => {
    if (tabSlug?.startsWith("actions")) {
      tabsRef.value?.selectTab(tabSlug);
    }
  },
);

watch(
  [() => changeSetsStore.selectedChangeSetLastWrittenAt],
  () => {
    if (
      componentsStore.selectedComponent &&
      componentsStore.selectedComponent.changeStatus !== "deleted"
    ) {
      actionsStore.FETCH_COMPONENT_ACTIONS(props.componentId);
    }
  },
  { immediate: true },
);
</script>
