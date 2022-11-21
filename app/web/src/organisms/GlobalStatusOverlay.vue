<template>
  <ProgressBarOverlay
    :title="summaryMessage"
    :detail="detailMessage"
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

const detailMessage = computed(() => {
  if (!statusStore.globalStatus?.isUpdating) return;
  const latestUpdate = statusStore.latestComponentUpdate;
  if (!latestUpdate) return undefined;
  return `${latestUpdate.statusMessage} - component ${latestUpdate.componentId}`;
});

const summaryMessage = computed(() => {
  // TODO: extra logic to show "update completed" for a timeout before flipping back to idle
  return statusStore.globalStatus?.isUpdating
    ? "Updating & testing the model"
    : "Model is up to date";
});

const progressPercent = computed(() => {
  if (statusStore.globalStatus?.stepsCountTotal === undefined) return undefined;
  return (
    (statusStore.globalStatus?.stepsCountCurrent || 0) /
    (statusStore.globalStatus?.stepsCountTotal || 1)
  );
});
</script>
