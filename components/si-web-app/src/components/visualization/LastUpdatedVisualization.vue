<template>
  <div>
    <div class="flex text-sm text-gray-400">
      <div class="font-normal">last updated:</div>
      <div class="font-bold">2d ago</div>
    </div>

    <div class="w-16">
      <line-chart
        class="relative"
        :height="100"
        :chart-data="datacollection"
        :options="chartOptions"
      />
    </div>
  </div>
</template>

<script>
import LineChart from "@/components/visualization/charts/LineChart.vue"
export default {
  name: "LastUpdatedVisualization",
  components: {
    LineChart
  },
  data () {
    return {
      datacollection: {},
      chartOptions: {
        legend: {
          display: false,
        },
        scales: {
          xAxes: [{
            display: false
          }],
          yAxes: [{
            display: false
          }],
        }
        // responsive: false
      },
    }
  },
  mounted () {
    this.fillData()
  },
  methods: {
    fillData () {
      this.datacollection = {
        labels: this.getData(),
        datasets: [
          {
            label: 'Data One',
            backgroundColor: '#FFDA06',
            lineTension: 0,
            pointRadius: 0,
            data: this.getData()
          }
        ]
      }
    },
    getData () {
      return [3,2,4,3,3,5,4,6,5,4,3,5,3]
    },
  }
}

</script>