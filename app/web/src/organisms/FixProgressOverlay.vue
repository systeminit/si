<template>
  <div
    :class="
      clsx(
        'absolute z-20 left-0 right-0 mx-4 mt-3 p-4',
        'bg-white dark:bg-neutral-800 dark:text-white border border-neutral-300 dark:border-neutral-600',
        'shadow-md rounded-md font-bold',
      )
    "
  >
    <div class="flex justify-between items-center">
      <span>
        {{ fixState.summary }}
        <span v-if="fixState.highlightedSummary" class="text-destructive-500">
          {{ fixState.highlightedSummary }}
        </span>
      </span>
    </div>

    <Transition
      enter-active-class="duration-300 ease-out"
      enter-from-class="transform opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="delay-1000 duration-200 ease-in"
      leave-from-class="opacity-100 "
      leave-to-class="transform opacity-0"
    >
      <div v-show="fixState.rate < 1" class="mt-2 flex gap-5">
        <ProgressBar :completion-rate="fixState.rate" />
        <span class="whitespace-nowrap flex-shrink-0">
          {{ fixState.executed }} of
          <span v-if="fixState.total > 0">
            {{ fixState.total }}
          </span>
          <Icon
            v-else
            name="loader"
            size="xs"
            class="inline-block align-middle"
          />
          {{ fixState.mode === "syncing" ? "Synced" : "Fixed" }}
        </span>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { computed } from "vue";
import ProgressBar from "@/atoms/ProgressBar.vue";
import { useFixesStore } from "@/store/fixes/fixes.store";
import Icon from "@/ui-lib/icons/Icon.vue";

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
    const rate = executed / total;

    return {
      mode: "fixing",
      rate,
      executed,
      total,
      summary: total === executed ? "Fixes applied!" : "Applying fixes...",
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

    let summary = "Determining fixes for updated model...";
    let highlightedSummary = "";
    if (rate === 1) {
      summary = "Model up-to-date";
      if (fixesStore.unstartedRecommendations.length !== 0) {
        summary += " - ";
        highlightedSummary = `${fixesStore.unstartedRecommendations.length} Resources need to be fixed`;
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
