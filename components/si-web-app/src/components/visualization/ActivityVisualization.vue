<template>
  <div>
    <div class="flex text-sm text-gray-400 font-bold">Activity</div>

    <div class="flex flex-col">
      <div class="flex">
        <div class="text-xs font-normal chart-title-service">services</div>
        <div class="ml-5 w-16">
          <line-chart
            class="relative"
            :height="100"
            :chart-data="datacollectionService"
            :options="chartOptions"
          />
        </div>
      </div>

      <div class="flex">
        <div class="text-xs font-normal chart-title-resources">resources</div>
        <div class="ml-3 w-16">
          <line-chart
            class="relative"
            :height="100"
            :chart-data="datacollectionResource"
            :options="chartOptions"
          />
        </div>
      </div>

    </div>
  </div>
</template>

<script>
import LineChart from "@/components/visualization/charts/LineChart.vue"
export default {
  name: "ActivityVisualization",
  components: {
    LineChart
  },
  data () {
    return {
      datacollectionService: {},
      datacollectionResource: {},
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
      this.datacollectionService = {
        labels: this.getData(),
        datasets: [
          {
            label: 'Data One',
            borderColor: '#0092B4',
            lineTension: 0,
            pointRadius: 0,
            fill: false,
            borderWidth: 1,
            data: this.getData()
          }
        ]
      },
      this.datacollectionResource = {
        labels: this.getData(),
        datasets: [
          {
            label: 'Data One',
            borderColor: '#B88100',
            lineTension: 0,
            pointRadius: 0,
            fill: false,
            borderWidth: 1,
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

<style>
.chart-title-service {
  color: #0092B4
}

.chart-title-resources {
  color: #B88100
}

</style>