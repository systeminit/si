<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      services
    </div>

    <div class="flex mt-1 pl-1">
      <div
        v-for="service in services"
        class="flex flex-row mr-1"
        :key="service.id"
      >
        <SquareChart :rgbColor="serviceColor" />
        <SquareChart :width="7" :rgbColor="statusColor" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import SquareChart from "@/components/visualization/charts/SquareChart.vue";
import _ from "lodash";

import { ServiceEntity } from "@/graphql-types";

export default Vue.extend({
  name: "ServicesVisualization",
  components: {
    SquareChart,
  },
  props: {
    applicationId: String,
  },
  data() {
    return {
      serviceColor: "78,141,171",
    };
  },
  computed: {
    services(): ServiceEntity[] {
      const edges = this.$store.getters["edge/fromIdForType"]({
        id: this.applicationId,
        typeName: "service_entity",
      });
      const results: ServiceEntity[] = _.filter(
        this.$store.state.entity.entities,
        (entity: ServiceEntity) => {
          for (const edge of edges) {
            if (edge.properties.headVertex.typeName == "service_entity") {
              return entity.id == edge.properties.headVertex.id;
            } else if (
              edge.properties.tailVertex.typeName == "service_entity"
            ) {
              return entity.id == edge.properties.tailVertex.id;
            } else {
              return false;
            }
          }
        },
      );
      return results;
    },
    statusColor(): string {
      let stateSuccessColor = "0,179,79";
      let stateFailureColor = "187,107,0";
      let colors = [stateSuccessColor, stateFailureColor];
      return colors[Math.floor(Math.random() * 2)];
    },
  },
});
</script>
