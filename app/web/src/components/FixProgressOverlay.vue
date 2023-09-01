<template>
  <ProgressBarOverlay
    v-if="fixState"
    :title="fixState.summary"
    :detail="fixState.highlightedSummary"
    :doneCount="fixState.executed"
    :totalCount="fixState.total"
    :barLabel="fixState.mode === 'syncing' ? 'Synced' : 'Applied'"
    :showRefreshButton="false"
  />
  <GlobalStatusOverlay v-else />
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useFixesStore } from "@/store/fixes.store";
import ProgressBarOverlay from "@/components/ProgressBarOverlay.vue";
import GlobalStatusOverlay from "@/components/GlobalStatusOverlay.vue";

const fixesStore = useFixesStore();

const fixState = computed(() => {
  if (fixesStore.runningFixBatch) {
    const total = fixesStore.fixesOnRunningBatch.length;
    const executed = fixesStore.completedFixesOnRunningBatch.length;

    let summary = "Applying actions...";
    if (total > 0 && total === executed) {
      summary = "Finishing up actions...";
    }

    return {
      mode: "fixing",
      executed,
      total,
      summary,
      highlightedSummary: "",
    };
  }
  return null;
});
</script>
