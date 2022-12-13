<template>
  <ProgressBarOverlay
    :title="statusStore.globalStatusMessage"
    :detail="statusStore.globalStatusDetailMessage"
    :done-count="statusStore.globalStatus?.componentsCountCurrent"
    :total-count="statusStore.globalStatus?.componentsCountTotal"
    bar-label="Updated"
    :progress-percent="progressPercent"
  />
</template>

<script setup lang="ts">
import { computed } from "vue";
import ProgressBarOverlay from "@/molecules/ProgressBarOverlay.vue";
import { useStatusStore } from "@/store/status.store";

const statusStore = useStatusStore();

const progressPercent = computed(() => {
  if (!statusStore.globalStatus?.stepsCountTotal) return undefined;
  return (
    (statusStore.globalStatus?.stepsCountCurrent || 0) /
    (statusStore.globalStatus?.stepsCountTotal || 1)
  );
});
</script>
