<template>
  <ProgressBarOverlay
    v-if="actionState"
    :title="actionState.summary"
    :detail="actionState.highlightedSummary"
    :doneCount="actionState.executed"
    :totalCount="actionState.total"
    :barLabel="actionState.mode === 'syncing' ? 'Synced' : 'Applied'"
    :showRefreshButton="false"
  />
  <GlobalStatusOverlay v-else />
</template>

<script lang="ts" setup>
// TODO(WENDY) - this old version of that horrible bar at the top of the diagram should be removed once we're done reproducing its functionality elsewhere

import { computed } from "vue";
import { useActionsStore } from "@/store/actions.store";
import ProgressBarOverlay from "@/components/ProgressBarOverlay.vue";
import GlobalStatusOverlay from "@/components/GlobalStatusOverlay.vue";

const actionsStore = useActionsStore();

const actionState = computed(() => {
  if (actionsStore.runningActionBatch) {
    const total = actionsStore.actionsOnRunningBatch.length;
    const executed = actionsStore.completedActionsOnRunningBatch.length;

    let summary = "Applying actions...";
    if (total > 0 && total === executed) {
      summary = "Finishing up actions...";
    }

    return {
      mode: "acting",
      executed,
      total,
      summary,
      highlightedSummary: "",
    };
  }
  return null;
});
</script>
