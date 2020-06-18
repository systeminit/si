<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
    services
    </div>
    
    <div class="flex">
      <div v-for="n in 3" class="mr-1" :key="n">
        <SquareChart
          rgbColor="6,142,255"
        />
      </div>
    </div>
  </div>
</template>

<script>
import SquareChart from "@/components/visualization/charts/SquareChart.vue"
export default {
  name: "ServicesVisualization",
  components: {
    SquareChart
  }
}

</script>