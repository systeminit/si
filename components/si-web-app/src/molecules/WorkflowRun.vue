<template>
  <div>
    <!-- <VueJsonPretty :data="workflowRun" /> -->
    <div class="flex flex-col mb-2 select-text">
      <!-- <VueJsonPretty :data="data" /> -->

      <div class="flex items-center justify-between px-2 py-1 title-bar">
        <div class="flex flex-row items-center text-sm">
          <div>
            {{ formatName(workflowRun.data.name) }}
          </div>

          <div
            class="ml-1 text-xs text-gray-200"
            v-if="workflowRun.endTimestamp"
          >
            {{ formatTimestamp(workflowRun.endTimestamp) }}
          </div>
          <div class="ml-1 text-xs text-gray-200" v-else>
            {{ formatTimestamp(workflowRun.startTimestamp) }}
          </div>
        </div>

        <div class="flex flex-row">
          <div class="text-xs" :class="stateClasses()">
            {{ workflowRunState }}
          </div>

          <button @click="toggleSummary()" class="ml-2 focus:outline-none">
            <ChevronDownIcon
              v-if="summaryVisible() || didRun"
              size="1.1x"
              class="text-gray-300 "
            />
            <ChevronRightIcon size="1.1x" v-else class="text-gray-300 " />
          </button>
        </div>
      </div>

      <div
        class="flex flex-row justify-between px-2 py-2 summary-bar"
        v-show="summaryVisible() || didRun"
      >
        <div class="flex flex-col">
          <div class="flex flex-row text-xs">
            <div class="">
              Start:
            </div>
            <div class="ml-1">
              {{ workflowRun.startTimestamp }}
            </div>
          </div>

          <div class="flex flex-row text-xs" v-if="workflowRun.endTimestamp">
            <div class="">
              Finish:
            </div>
            <div class="ml-1">
              {{ workflowRun.endTimestamp }}
            </div>
          </div>
        </div>

        <div
          class="flex flex-row items-end text-xs"
          v-if="workflowRun.endTimestamp"
        >
          <div class="">
            Total:
          </div>
          <div class="ml-1">
            {{
              timeDelta(workflowRun.startTimestamp, workflowRun.endTimestamp)
            }}
          </div>
        </div>
      </div>

      <div v-show="summaryVisible() || didRun">
        <WorkflowStep
          v-for="(data, index) in workflowSteps"
          :key="index"
          :step="data.step"
          :stepEntities="data.stepEntities"
          :expanded="isExpanded"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import _ from "lodash";

import { SiTime } from "@/api/sit";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";

import { IListActionReplySuccess } from "@/api/sdf/dal/workflowDal";

import { WorkflowRun, WorkflowRunState } from "@/api/sdf/model/workflow";

import WorkflowStep from "@/molecules/WorkflowStep.vue";

// import VueJsonPretty from "vue-json-pretty";

interface IData {
  isSummaryVisible: boolean;
  didRun: boolean;
  isRunning: boolean;
  isExpanded: boolean;
}

export default Vue.extend({
  name: "WorkflowRun",
  components: {
    ChevronDownIcon,
    ChevronRightIcon,
    WorkflowStep,
    // VueJsonPretty,
  },
  props: {
    workflowRun: {
      type: Object as PropType<WorkflowRun>,
      required: true,
    },
    workflowSteps: {
      // @ts-ignore
      type: Array as PropType<IListActionReplySuccess["workflowRuns"]["steps"]>,
      required: true,
    },
  },
  data(): IData {
    return {
      isSummaryVisible: false,
      isRunning: false,
      didRun: false,
      isExpanded: false,
    };
  },
  computed: {
    workflowRunState(): string {
      if (String(this.workflowRun.state) == "invoked") {
        return "running";
      } else {
        return this.workflowRun.state;
      }
    },
  },
  methods: {
    formatTimestamp(timestamp: string): String {
      const date = new Date(timestamp);
      const year = date.getFullYear();
      const month = date.getMonth();
      const day = date.getDay();
      const hour = date.getHours();
      const minute = date.getMinutes();
      return `(${month}/${day}/${year} - ${hour}:${minute})`;
    },
    formatName(name: string): String {
      return _.capitalize(name);
    },
    timeDelta(t1: string, t2: string): string {
      return SiTime.timeDelta(t1, t2);
    },
    toggleSummary() {
      if (this.didRun) {
        this.isSummaryVisible = true;
      }
      this.isSummaryVisible = !this.isSummaryVisible;
      this.didRun = false;
    },
    summaryVisible() {
      if (this.workflowRunState == "running") {
        return true;
      } else {
        return this.isSummaryVisible;
      }
    },
    stateClasses(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (this.workflowRun.state == "success") {
        classes["state-succeeded"] = true;
      } else if (this.workflowRunState == "running") {
        classes["state-running"] = true;
      } else {
        classes["state-failed"] = true;
      }
      return classes;
    },
  },
  watch: {
    workflowRunState(state: string) {
      if (state == "running") {
        this.isRunning = true;
        this.didRun = true;
      } else {
        if (this.isRunning) {
          this.didRun = true;
          this.isRunning = false;
        }
      }
      if (this.didRun || this.isRunning) {
        this.isExpanded = true;
      }
    },
  },
});
</script>

<style scoped>
.title-bar {
  background-color: #2a2c2d;
}

.summary-bar {
  background-color: #313536;
}

.state-succeeded {
  color: #00ff6f;
}

.state-failed {
  color: #ff003d;
}

.state-running {
  color: #00e4ff;
}
</style>
