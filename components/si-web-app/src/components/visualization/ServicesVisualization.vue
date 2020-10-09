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
        <SquareChart
          :applicationId="applicationId"
          :inEditor="inEditor"
          :service="service"
          :rgbColor="serviceColor"
        />
      </div>
    </div>
    <div class="flex justify-end w-full pt-3 pr-1" v-if="inEditor == 'true'">
      <Button2
        @click.native="deployApplication"
        label="deploy"
        icon="deploy"
        kind="standard"
        size="xs"
        :disabled="!changeSet"
      />
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import { RootStore } from "@/store";
import SquareChart from "@/components/visualization/charts/SquareChart.vue";
import Button2 from "@/components/ui/Button2.vue";

export default Vue.extend({
  name: "ServicesVisualization",
  components: {
    SquareChart,
    Button2,
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
    ...mapState({
      changeSet(state: RootStore): RootStore["editor"]["changeSet"] {
        return state.editor.changeSet;
      },
    }),
    objects(): Record<string, Entity> {
      if (this.inEditor) {
        return this.$store.state.editor.objects;
      } else {
        return this.$store.state.application.resources[this.applicationId];
      }
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
  methods: {
    async deployApplication() {
      let nodeId = this.$store.state.editor.application?.nodeId;
      if (nodeId) {
        await this.$store.dispatch("editor/entityAction", {
          action: "deploy",
          nodeId,
        });
        await this.$store.dispatch("editor/changeSetExecute");
        this.$store.commit("editor/setMode", "view");
      }
    },
  },
});
</script>
