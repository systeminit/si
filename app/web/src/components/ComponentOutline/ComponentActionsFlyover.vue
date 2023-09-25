<template>
  <div
    :class="
      clsx(
        'bg-neutral-700 w-96 h-96 rounded flex flex-col overflow-clip text-white shadow-3xl dark',
        themeContainerClasses,
      )
    "
  >
    <div
      class="bg-black uppercase font-bold text-md pt-sm pb-xs px-xs shrink-0"
    >
      <span>Actions - {{ component?.displayName }}</span>
    </div>
    <TabGroup as="template">
      <TabList
        class="bg-black flex font-bold text-sm children:uppercase children:border-b children:border-transparent children:px-xs children:py-xs"
      >
        <Tab class="ui-selected:border-action-300 ui-selected:text-action-300">
          Select
        </Tab>
        <Tab class="ui-selected:border-action-300 ui-selected:text-action-300">
          History
          <PillCounter class="ml-2xs" :count="filteredBatches.length" />
        </Tab>
      </TabList>
      <TabPanels as="template">
        <TabPanel class="overflow-auto grow">
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
        </TabPanel>
        <TabPanel class="overflow-auto grow">
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
        </TabPanel>
      </TabPanels>
    </TabGroup>
  </div>
</template>

<script setup lang="ts">
import { computed, PropType } from "vue";
import * as _ from "lodash-es";

import { PillCounter, useThemeContainer } from "@si/vue-lib/design-system";
import { TabGroup, TabList, Tab, TabPanels, TabPanel } from "@headlessui/vue";
import clsx from "clsx";
import { ComponentId, useComponentsStore } from "@/store/components.store";

import { useFixesStore } from "@/store/fixes.store";

import ApplyHistoryItem from "@/components/ApplyHistoryItem.vue";
import { useActionsStore } from "@/store/actions.store";
import EmptyStateIcon from "../EmptyStateIcon.vue";
import ActionWidget from "../ActionWidget.vue";

const { themeContainerClasses } = useThemeContainer("dark");

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const componentsStore = useComponentsStore();
const fixesStore = useFixesStore();
const actionsStore = useActionsStore();

const component = computed(
  () => componentsStore.componentsById[props.componentId],
);

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
