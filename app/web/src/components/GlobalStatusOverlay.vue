<template>
  <ProgressBarOverlay
    :title="statusStore.globalStatusMessage"
    :detail="statusStore.globalStatusDetailMessage"
    :doneCount="statusStore.globalStatus.updatedComponents"
    :totalCount="statusStore.globalStatus.totalComponents"
    barLabel="Updated"
    :progressPercent="progressPercent"
  />
</template>

<script setup lang="ts">
import { computed } from "vue";
import ProgressBarOverlay from "@/components/ProgressBarOverlay.vue";
import { useStatusStore } from "@/store/status.store";

const statusStore = useStatusStore();

const progressPercent = computed(
  () =>
    (statusStore.globalStatus.totalComponents || 1) /
    (statusStore.globalStatus.updatedComponents || 1),
);
</script>
