<template>
  <div class="w-full h-full">
    <SummaryCard>
      <template #title>Activity</template>

      <template #content>
        <div class="w-full h-full">
          <!--
        <div :style="canvasStyle">
          <canvas :id="canvasId" />
          </div>
          -->
        </div>
      </template>
    </SummaryCard>
  </div>
</template>

<script setup lang="ts">
import SummaryCard from "@/atoms/SummaryCard.vue";
// import { ChartConfiguration, Chart } from "chart.js";

defineProps({
  showChartLabels: {
    type: Boolean,
    default: true,
  },
  height: {
    type: String,
    default: "60px",
  },
  applicationId: {
    type: Number,
    required: true,
  },
});
</script>

<style scoped>
.canvas-container {
  height: 60px;
  width: 100%;
}

.details-panel {
  border: solid;
  border-width: 1px;
  border-color: #464753;
  background-color: #101010;
}

.details-panel-title {
  /* @apply font-normal text-xs; */
  font-weight: 400;
  font-size: 0.75rem;
  line-height: 1rem;
}

.details-panel-background {
  background-color: #171717;
}
</style>
