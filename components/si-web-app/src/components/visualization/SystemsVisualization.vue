<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      systems
    </div>
    <div class="flex mt-1 pl-1">
      <div v-for="n in 2" class="mr-1" :key="n">
        <SquareChart rgbColor="115, 76, 118" />
      </div>
    </div>
  </div>
</template>

<script>
import SquareChart from "@/components/visualization/charts/SquareChart.vue";
export default {
  name: "SystemsVisualization",
  components: {
    SquareChart,
  },
};
</script>
