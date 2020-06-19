<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
    services
    </div>
    
    <div class="flex mt-1 pl-1">
      <div v-for="n in 3" class="flex flex-row mr-1" :key="n">
        <SquareChart
          :rgbColor="serviceColor"
        />
        <SquareChart
          :width=7
          :rgbColor="statusColor"
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
  },
  data() {
    return {
      serviceColor: "78,141,171",
    }
  },
  computed: {
    statusColor() {
      let stateSuccessColor = "0,179,79"
      let stateFailureColor = "187,107,0"
      let colors = [stateSuccessColor, stateFailureColor]
      return colors[Math.floor(Math.random() * 2)]
    }
  }
}

</script>