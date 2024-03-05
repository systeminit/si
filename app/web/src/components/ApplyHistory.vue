<template>
  <div v-if="actionBatches.length === 0" class="flex flex-col items-center">
    <div class="w-52">
      <EmptyStateIcon name="actions" />
    </div>
    <div class="text-xl text-neutral-400 dark:text-neutral-300 mt-2">
      No Actions To Be Taken
    </div>
    <div class="text-sm px-xs pt-3 text-neutral-400 text-center italic">
      There are no <span class="font-bold">actions</span> to display for the
      selected asset(s)
    </div>
  </div>
  <ScrollArea v-else>
    <!-- TODO(Wendy) - this search bar isn't implemented, so removing it for now -->
    <!-- <template #top>
      <SiSearch autoSearch />
    </template> -->

    <ApplyHistoryItem
      v-for="(actionBatch, index) in actionBatches"
      :key="index"
      :actionBatch="actionBatch"
      :collapse="index !== 0"
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { ScrollArea } from "@si/vue-lib/design-system";
// import SiSearch from "@/components/SiSearch.vue";
import { useActionsStore } from "@/store/actions.store";
import ApplyHistoryItem from "@/components/ApplyHistoryItem.vue";
import EmptyStateIcon from "./EmptyStateIcon.vue";

const actionsStore = useActionsStore();

const actionBatches = computed(() =>
  _.reverse([...actionsStore.actionBatches]),
);
</script>
