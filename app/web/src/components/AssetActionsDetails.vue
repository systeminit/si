<template>
  <div class="h-full relative">
    <TabGroup minimal>
      <TabGroupItem label="Select" slug="action-selection">
        <div
          v-if="actions.length === 0"
          class="flex flex-col items-center pt-lg h-full w-full text-neutral-400"
        >
          <div class="w-64">
            <EmptyStateIcon name="no-changes" />
          </div>
          <span class="text-xl">No Actions available</span>
        </div>
        <div v-else class="flex flex-col p-xs gap-xs">
          <div class="text-sm text-neutral-300">
            Actions will be enacted upon click of the
            <b>APPLY CHANGES</b> button in the right rail.
          </div>
          <ActionWidget
            v-for="action in actions"
            :key="action.actionPrototypeId"
            :componentId="componentId"
            :actionPrototypeId="action.actionPrototypeId"
          />
        </div>
      </TabGroupItem>
      <TabGroupItem slug="action-history">
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
        <ul v-else class="flex flex-col gap-2xs p-xs">
          <li
            v-for="(fixBatch, index) in filteredBatches"
            :key="index"
            class="bg-black p-xs"
          >
            <ApplyHistoryItem :fixBatch="fixBatch" />
          </li>
        </ul>
      </TabGroupItem>
    </TabGroup>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import * as _ from "lodash-es";

import { Inline, TabGroup, TabGroupItem } from "@si/vue-lib/design-system";

import { ComponentId } from "@/store/components.store";

import { useFixesStore } from "@/store/fixes.store";

import ApplyHistoryItem from "@/components/ApplyHistoryItem.vue";
import { useActionsStore } from "@/store/actions.store";
import EmptyStateIcon from "@/components/EmptyStateIcon.vue";
import ActionWidget from "@/components/ActionWidget.vue";

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const fixesStore = useFixesStore();
const actionsStore = useActionsStore();

const actions = computed(
  () => actionsStore.actionsByComponentId[props.componentId] || [],
);

const fixBatches = computed(() => _.reverse([...fixesStore.fixBatches]));

const filteredBatches = computed(() =>
  fixBatches.value
    .map((batch) => ({
      ...batch,
      fixes: batch.fixes.filter((fix) => fix.componentId === props.componentId),
    }))
    .filter((batch) => batch.fixes.length),
);
</script>
