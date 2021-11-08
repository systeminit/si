<template>
  <div class="flex flex-col px-1 mt-1">
    <!-- <VueJsonPretty :data="workflowStep" /> -->

    <div class="flex flex-row justify-between summary">
      <div class="flex flex-col">
        <div class="flex flex-row text-xs">
          <div class="">
            Start:
          </div>
          <div class="ml-1">
            {{ stepEntity.startTimestamp }}
          </div>
        </div>

        <div class="flex flex-row mt-1 text-xs" v-if="stepEntity.endTimestamp">
          <div class="">
            Finish:
          </div>
          <div class="ml-1">
            {{ stepEntity.endTimestamp }}
          </div>
        </div>
      </div>

      <div
        class="flex flex-row items-end text-xs"
        v-if="stepEntity.endTimestamp"
      >
        <div class="">
          Total:
        </div>
        <div class="ml-1">
          {{ timeDelta(stepEntity.startTimestamp, stepEntity.endTimestamp) }}
        </div>
      </div>
    </div>

    <div class="my-3 separator" />

    <div
      class="flex flex-col text-xs antialiased font-light leading-tight tracking-tight text-left text-gray-200 whitespace-pre-wrap cmd-output"
    >
      <div v-if="stepEntity.output">
        {{ stepEntity.output }}
      </div>

      <div class="mt-2 failures" v-if="stepEntity.error">
        {{ stepEntity.error }}
      </div>
    </div>

    <!-- <VueJsonPretty :data="stepEntity" /> -->
  </div>
</template>

<script lang="ts">
/* eslint-disable vue/no-unused-components */

import Vue, { PropType } from "vue";

import { SiTime } from "@/api/sit";

import VueJsonPretty from "vue-json-pretty";

import { WorkflowRunStepEntity } from "@/api/sdf/model/workflow";

interface IData {
  isSummaryVisible: boolean;
}

export default Vue.extend({
  name: "WorkflowStepEntity",
  components: {
    VueJsonPretty,
  },
  props: {
    stepEntity: {
      type: Object as PropType<WorkflowRunStepEntity>,
      required: true,
    },
    expanded: {
      type: Boolean,
      default: false,
    },
  },
  data(): IData {
    return {
      isSummaryVisible: this.expanded,
    };
  },
  methods: {
    timeDelta(t1: string, t2: string): string {
      return SiTime.timeDelta(t1, t2);
    },
  },
});
</script>

<style scoped>
.failures {
  color: #ff7651;
}

.summary {
  color: #c6daef;
}

.cmd-output {
  color: #ffefb2;
}
.separator {
  height: 1px;
  background-color: #323637;
}
</style>
