<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      systems
    </div>
    <div class="flex mt-1 pl-1">
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
import { mapGetters } from "vuex";

import { SystemStore } from "@/store/modules/system";
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
    systems(): SystemStore["systems"][] {
      return this.$store.getters["system/forApplicationId"](this.applicationId);
    },
  },
});
</script>
