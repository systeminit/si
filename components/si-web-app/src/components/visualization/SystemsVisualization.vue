<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      systems
    </div>
    <div class="flex">
      <div v-for="n in 2" class="mr-1" :key="n">
        <SquareChart
          rgbColor="255,64,161"
        />
      </div>
    </div>
  </div>
</template>

<script>
import SquareChart from "@/components/visualization/charts/SquareChart.vue"
export default {
  name: "SystemsVisualization",
  components: {
    SquareChart
  }
}

</script>