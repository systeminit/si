<template>
  <ProgressBarOverlay
    v-if="fixesStore.runningFixBatch"
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
const loadConfirmationsReqStatus =
  fixesStore.getRequestStatus("LOAD_CONFIRMATIONS");
const _execFixesReqStatus = fixesStore.getRequestStatus(
  "EXECUTE_FIXES_FROM_RECOMMENDATIONS",
);

const fixState = computed(() => {
  if (fixesStore.runningFixBatch) {
    const total = fixesStore.fixesOnRunningBatch.length;
    const executed = fixesStore.completedFixesOnRunningBatch.length;

    // Reload confirmations if all recommendations have been applied!
    let summary = "Applying actions...";
    if (total > 0 && total === executed) {
      summary = "Finishing up actions...";
      fixesStore.LOAD_CONFIRMATIONS();
    }

    return {
      mode: "fixing",
      executed,
      total,
      summary,
      highlightedSummary: "",
    };
  } else {
    let rate = 0;
    const finishedConfirmations = fixesStore.finishedConfirmations.length;
    const numberOfConfirmations = fixesStore.confirmations.length;
    if (loadConfirmationsReqStatus.value.isSuccess) {
      if (fixesStore.confirmations.length > 0) {
        rate = finishedConfirmations / numberOfConfirmations;
      } else {
        rate = 1;
      }
    }

    let summary = "Determining recommendations for updated model...";
    let highlightedSummary = "";
    if (rate === 1) {
      summary = "Recommendations are up to date.";
      const { length } = fixesStore.newRecommendations;

      if (length !== 0) {
        highlightedSummary = `${length} recommendation${
          length > 1 ? "s" : ""
        } pending`;
      }
    }

    return {
      mode: "syncing",
      rate,
      executed: finishedConfirmations,
      total: numberOfConfirmations,
      summary,
      highlightedSummary,
    };
  }
});
</script>
