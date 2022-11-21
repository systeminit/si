<template>
  <ProgressBarOverlay
    :title="fixState.summary"
    :detail="fixState.highlightedSummary"
    :done-items="fixState.executed"
    :total-items="fixState.total"
    :bar-label="fixState.mode === 'syncing' ? 'Synced' : 'Applied'"
  />
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useFixesStore } from "@/store/fixes/fixes.store";
import ProgressBarOverlay from "@/molecules/ProgressBarOverlay.vue";

const fixesStore = useFixesStore();
const loadConfirmationsReqStatus =
  fixesStore.getRequestStatus("LOAD_CONFIRMATIONS");
const _execFixesReqStatus = fixesStore.getRequestStatus(
  "EXECUTE_FIXES_FROM_RECOMMENDATIONS",
);

const fixState = computed(() => {
  if (fixesStore.runningFixBatch) {
    const total = fixesStore.fixesOnRunningBatch.length;
    const executed = fixesStore.fixesOnRunningBatch.length;
    console.log(total);
    console.log(executed);

    return {
      mode: "fixing",
      executed,
      total,
      summary:
        total === executed
          ? "Recommendations applied!"
          : "Applying recommendations...",
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
      summary = "Model is up-to-date";
      const { length } = fixesStore.unstartedRecommendations;
      if (length !== 0) {
        summary += " - ";
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
