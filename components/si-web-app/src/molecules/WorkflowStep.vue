<template>
  <div class="flex flex-col pl-4 pr-2 background">
    <div
      class="flex flex-row items-center justify-between my-1 text-xs title-bar"
      :class="titleClasses()"
    >
      <div class="flex flex-row">
        <div>{{ step.step.kind }} {{ step.step.inputs.name.value }}</div>
      </div>

      <button @click="toggleSummary()" class="ml-2 focus:outline-none">
        <ChevronDownIcon v-if="isSummaryVisible" size="1.1x" />
        <ChevronRightIcon size="1.1x" v-else />
      </button>
    </div>

    <div v-show="isSummaryVisible" class="pt-1 pb-3">
      <WorkflowStepEntity
        v-for="(data, index) in stepEntities"
        ref="WorkflowStep"
        :key="index"
        :stepEntity="data"
        :expanded="expanded"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import {
  WorkflowRunStep,
  WorkflowRunStepEntity,
} from "@/api/sdf/model/workflow";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";

import WorkflowStepEntity from "@/molecules/WorkflowStepEntity.vue";

interface IData {
  isSummaryVisible: boolean;
}

export default Vue.extend({
  name: "WorkflowStep",
  components: {
    ChevronDownIcon,
    ChevronRightIcon,
    WorkflowStepEntity,
  },
  props: {
    step: {
      type: Object as PropType<WorkflowRunStep>,
      required: true,
    },
    stepEntities: {
      type: Array as PropType<WorkflowRunStepEntity[]>,
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
    titleClasses(): Record<string, any> {
      let classes: Record<string, any> = {};

      if (this.step.state == "success") {
        classes["state-succeeded"] = true;
      } else {
        classes["state-failed"] = true;
      }
      return classes;
    },
    toggleSummary() {
      this.isSummaryVisible = !this.isSummaryVisible;
    },
  },
});
</script>

<style scoped>
.background {
  background-color: #101112;
}

.title-bar {
  background-color: #141617;
}

.state-succeeded {
  color: #00ff6f;
}

.state-failed {
  color: #ff5a50;
}

.state-running {
  color: #00e4ff;
}
</style>
