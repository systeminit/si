<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      systems
    </div>
    <div class="flex pl-1 mt-1">
      <div v-for="system in systems" class="mr-1" :key="system.id">
        <SystemChart
          :system="system"
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
import SystemChart from "@/components/visualization/charts/SystemChart.vue";

export default Vue.extend({
  name: "SystemsVisualization",
  components: {
    SystemChart,
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
