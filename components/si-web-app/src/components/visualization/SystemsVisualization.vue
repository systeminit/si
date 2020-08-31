<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      systems
    </div>
    <div class="flex pl-1 mt-1">
      <div v-for="system in systems" class="mr-1" :key="system.id">
        <SquareChart
          rgbColor="115, 76, 118"
          :data-cy="'systems-visualization-' + system.name"
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";

import { RootStore } from "@/store";
import SquareChart from "@/components/visualization/charts/SquareChart.vue";

export default Vue.extend({
  name: "SystemsVisualization",
  components: {
    SquareChart,
  },
  props: {
    applicationId: String,
  },
  computed: {
    ...mapState({
      systems(state: RootStore) {
        return state.application.systems[this.applicationId];
      },
    }),
  },
});
</script>
