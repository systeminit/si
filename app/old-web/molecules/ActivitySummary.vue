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

import {
  buildActivityData,
  buildActivityDataSm,
} from "@/api/visualization/activityData";

import SummaryCard from "@/atoms/SummaryCard.vue";

import _ from "lodash";
import { ApplicationDal } from "@/api/sdf/dal/applicationDal";
import { workspace$, refreshActivitySummary$ } from "@/observables";
import { combineLatest } from "rxjs";
import { tap, pluck } from "rxjs/operators";
import { emitEditorErrorMessage } from "@/atoms/PanelEventBus";

interface IData {
  activityData: ChartConfiguration | null;
  activityDataSm: ChartConfiguration | null;
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
    applicationId: {
      type: String,
    },
  },
  subscriptions(): Record<string, any> {
    let applicationId$ = this.$watchAsObservable("applicationId", {
      immediate: true,
    }).pipe(pluck("newValue"));
    return {
      activityDataBackend: combineLatest(
        applicationId$,
        workspace$,
        refreshActivitySummary$,
      ).pipe(
        tap(async ([applicationId, workspace]) => {
          if (applicationId && workspace) {
            let reply = await ApplicationDal.activitySummary({
              applicationId,
              workspaceId: workspace.id,
            });
            if (reply.error) {
              emitEditorErrorMessage(reply.error.message);
            } else {
              // @ts-ignore
              this.activityData = buildActivityData(reply);
              // @ts-ignore
              this.activityDataSm = buildActivityDataSm(reply);
            }
            const ctx = document.getElementById(
              // @ts-ignore
              this.canvasId,
            ) as HTMLCanvasElement;

            // @ts-ignore
            if (this.showChartLabels) {
              // @ts-ignore
              new Chart(ctx, this.activityData);
            } else {
              // @ts-ignore
              new Chart(ctx, this.activityDataSm);
            }
          }
        }),
      ),
    };
  },
  data(): IData {
    let id = _.uniqueId("activity-summary:");
    return {
      activityData: null,
      activityDataSm: null,
      canvasId: id,
      canvasStyle: {
        height: this.height,
        width: "100%",
      },
    };
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
