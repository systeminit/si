<template>
  <div>
    <div class="text-sm font-bold text-gray-400">
      services
    </div>

    <div class="flex pl-1 mt-1">
      <div
        v-for="service in services"
        class="flex flex-row mr-1"
        :key="service.nodeId"
      >
        <SquareChart :rgbColor="serviceColor" />
        <SquareChart :width="7" :rgbColor="statusColor" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import SquareChart from "@/components/visualization/charts/SquareChart.vue";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";

export default Vue.extend({
  name: "ServicesVisualization",
  components: {
    SquareChart,
  },
  props: {
    applicationId: String,
    inEditor: String,
  },
  data() {
    return {
      serviceColor: "78,141,171",
    };
  },
  computed: {
    objects(): Record<string, Entity> {
      return this.$store.state.editor.objects;
    },
    services(): Entity[] {
      const results: Entity[] = [];
      if (this.inEditor == "true") {
        for (const entityId of Object.keys(this.objects)) {
          const entity: Entity = this.objects[entityId];
          if (entity.objectType == "service" && !entity.siStorable.deleted) {
            results.push(entity);
          }
        }
        return results;
      } else {
        let appEntity = _.find(this.$store.state.application.list, [
          "id",
          this.applicationId,
        ]);
        if (appEntity) {
          return (
            _.filter(this.$store.state.application.services[appEntity.nodeId], [
              "siStorable.deleted",
              false,
            ]) || []
          );
        }
      }
      return [];
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
