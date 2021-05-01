<template>
  <SummaryCard>
    <template v-slot:title>Computing Resources</template>

    <template v-slot:content>
      <div class="flex flex-col w-full h-full">
        <div class="flex flex-row flex-wrap w-full h-full mx-1">
          <div
            class="mr-2"
            v-for="(computingResource, index) in computingResourcesData"
            :key="index"
          >
            <ResourceVisualization :data="computingResource" />
          </div>
        </div>

        <div class="flex justify-end mt-2" v-show="showButton">
          <div class="flex items-center justify-center button">
            <div class="mx-1 align-middle button-text">Sync</div>
          </div>
        </div>
      </div>
    </template>
  </SummaryCard>
</template>

<script lang="ts">
import Vue from "vue";

import {
  computingResourceData,
  ComputingResource,
} from "@/api/visualization/computingResourcesData";
import ResourceVisualization from "@/molecules/ComputingResourceSummary/ComputingResourceVisualization.vue";

import SummaryCard from "@/atoms/SummaryCard.vue";

interface IData {
  computingResourcesData: ComputingResource[];
}

export default Vue.extend({
  name: "ComputingResourceSummary",
  components: {
    ResourceVisualization,
    SummaryCard,
  },
  props: {
    showButton: {
      type: Boolean,
      default: true,
    },
  },
  data(): IData {
    return {
      computingResourcesData: computingResourceData,
    };
  },
});
</script>

<style lang="scss" scoped>
$button-saturation: 1.2;
$button-brightness: 1.05;

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

.button {
  background-color: #5a7b7c;
}

.button-text {
  @apply font-normal;
  font-size: 11px;
  margin-top: 2px;
  margin-bottom: 2px;
}

.button:hover {
  filter: brightness($button-brightness);
}

.button:focus {
  outline: none;
}

.button:active {
  filter: saturate(1.5) brightness($button-brightness);
}
</style>
