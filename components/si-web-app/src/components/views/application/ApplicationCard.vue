<template>
  <div class="mx-8 my-4 h-24 flex flex-row justify-between bg-blue-500">

    <div class="block mx-3 pt-3 text-white">
    {{ application.name }}
    </div>

    <div class="block mx-3 pt-3 w-24 h-full">
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
import LineChart from '@/components/charts/LineChart.js'

export default {
  name: "ApplicationCard",
  props: {
    application: {}
  },
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
    }
  }
}

</script>