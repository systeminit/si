<template>
  <ProgressBarOverlay
    :title="statusStore.globalStatusMessage"
    :detail="statusStore.globalStatusDetailMessage"
    :doneCount="statusStore.globalStatus?.componentsCountCurrent"
    :totalCount="statusStore.globalStatus?.componentsCountTotal"
    barLabel="Updated"
    :progressPercent="progressPercent"
  />
</template>

<script setup lang="ts">
import { computed } from "vue";
import ProgressBarOverlay from "@/components/ProgressBarOverlay.vue";
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
