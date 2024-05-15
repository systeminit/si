<template>
  <div
    v-if="
      (featureFlagsStore.IS_ACTIONS_V2 && !actions.length) ||
      (!featureFlagsStore.IS_ACTIONS_V2 && actionBatches.length === 0)
    "
    class="flex flex-col items-center"
  >
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
    <template v-if="featureFlagsStore.IS_ACTIONS_V2">
      <ActionRunnerCardV2
        v-for="action in actions"
        :key="action.id"
        :action="action"
      />
    </template>
    <template v-else>
      <ApplyHistoryItem
        v-for="(actionBatch, index) in actionBatches"
        :key="index"
        :actionBatch="actionBatch"
        :collapse="index !== 0"
      />
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import { ScrollArea } from "@si/vue-lib/design-system";
import { useActionsStore } from "@/store/actions.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ApplyHistoryItem from "@/components/ApplyHistoryItem.vue";
import ActionRunnerCardV2 from "@/components/ActionRunnerCardV2.vue";
import EmptyStateIcon from "./EmptyStateIcon.vue";

const featureFlagsStore = useFeatureFlagsStore();
const actionsStore = useActionsStore();

const actionBatches = computed(() => actionsStore.actionBatches);
const actions = computed(() => actionsStore.actionsV2);
</script>
