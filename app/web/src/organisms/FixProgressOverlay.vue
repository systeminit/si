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
    <div>
      {{ fixState.summary
      }}<span v-if="fixState.highlightedSummary" class="text-destructive-500">{{
        fixState.highlightedSummary
      }}</span>
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
          Synced
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
import Icon from "@/ui-lib/Icon.vue";

const fixesStore = useFixesStore();
const loadFixesReqStatus = fixesStore.getRequestStatus("LOAD_FIXES");
const execFixesReqStatus = fixesStore.getRequestStatus("EXECUTE_FIXES");

const fixState = computed(() => {
  if (fixesStore.runningFixBatch) {
    const total = fixesStore.fixesOnRunningBatch.length;
    const executed = fixesStore.completedFixesOnRunningBatch.length;
    let rate = 0;
    if (execFixesReqStatus.value.isSuccess && total > 0) {
      rate = executed / total;
    }

    return {
      rate,
      executed,
      total,
      summary: "Applying fixes...",
      highlightedSummary: "",
    };
  } else {
    let rate = 0;
    if (loadFixesReqStatus.value.isSuccess) {
      if (fixesStore.totalFixComponents > 0) {
        rate =
          fixesStore.processedFixComponents / fixesStore.totalFixComponents;
      } else {
        rate = 1;
      }
    }

    let summary = "Determining fixes for updated model...";
    let highlightedSummary = "";
    if (rate === 1) {
      summary = "Model up-to-date";
      if (fixesStore.unstartedFixes.length !== 0) {
        summary += " - ";
        highlightedSummary = `${fixesStore.unstartedFixes.length} Resources need to be fixed`;
      }
    }

    return {
      rate,
      executed: fixesStore.processedFixComponents,
      total: fixesStore.totalFixComponents,
      summary,
      highlightedSummary,
    };
  }
});
</script>
