<template>
  <div class="w-full h-full">
    <SummaryCard>
      <template v-slot:title>Activity</template>

      <template v-slot:content>
        <div class="w-full h-full">
          <div :style="canvasStyle">
            <canvas :id="canvasId" />
          </div>
        </div>
      </template>
    </SummaryCard>
  </div>
</template>

<script lang="ts">
import Vue from "vue";

import Chart from "chart.js";
import { ChartConfiguration } from "chart.js";

import { activityData, activityDataSm } from "@/api/visualization/activityData";

import SummaryCard from "@/atoms/SummaryCard.vue";

import _ from "lodash";

interface IData {
  activityData: ChartConfiguration;
  activityDataSm: ChartConfiguration;
  canvasId: string;
  canvasStyle: {
    height: string;
    width: string;
  };
}

export default Vue.extend({
  name: "ActivitySummary",
  components: {
    SummaryCard,
  },
  props: {
    showChartLabels: {
      type: Boolean,
      default: true,
    },
    height: {
      type: String,
      default: "60px",
    },
  },
  data(): IData {
    let id = _.uniqueId("activity-summary:");
    return {
      activityData: activityData,
      activityDataSm: activityDataSm,
      canvasId: id,
      canvasStyle: {
        height: this.height,
        width: "100%",
      },
    };
  },
  mounted() {
    const ctx = document.getElementById(this.canvasId) as HTMLCanvasElement;

    if (this.showChartLabels) {
      new Chart(ctx, this.activityData);
    } else {
      new Chart(ctx, this.activityDataSm);
    }
  },
  methods: {},
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
