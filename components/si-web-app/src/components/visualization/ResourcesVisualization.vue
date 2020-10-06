<template>
  <div class="flex flex-col h-full">
    <div class="text-sm font-bold text-gray-400">
      resources
    </div>

    <div class="flex pl-1 mt-1">
      <!-- <div
        v-for="service in services"
        class="flex flex-row mr-1"
        :key="service.id"
        > -->
      <div v-for="resource in resources" :key="resource.id" class="mr-1">
        <Tooltip>
          <CircleChart :resource="resource" />
          <template v-slot:tooltip>
            <div class="flex flex-col text-gray-400">
              <div class="text-sm">
                {{ resourceName(resource) }}
              </div>
              <div class="flex flex-col ml-2 text-xs">
                <div class="flex">
                  <div class="pr-2">
                    Kind
                  </div>
                  <div>
                    {{ resourceKind(resource) }}
                  </div>
                </div>
                <div class="flex">
                  <div class="pr-2">
                    Status
                  </div>
                  <div :style="statusStyle(resource)">
                    {{ resource.status }}
                  </div>
                </div>
                <div class="flex">
                  <div class="pr-2">
                    Health
                  </div>
                  <div :style="healthStyle(resource)">
                    {{ resource.health }}
                  </div>
                </div>
                <div class="flex">
                  <div class="pr-2">
                    Time
                  </div>
                  <div>
                    {{ resource.timestamp }}
                  </div>
                </div>
              </div>
            </div>
          </template>
        </Tooltip>
      </div>
    </div>
    <div class="flex content-end justify-end flex-grow w-full pt-2 pr-1">
      <div class="self-end">
        <Button2
          label="sync"
          icon="refresh"
          size="xs"
          @click.native="sync"
          disabled
          v-if="updating"
        />
        <Button2
          label="sync"
          icon="refresh"
          size="xs"
          @click.native="sync"
          v-else
        />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import CircleChart from "@/components/visualization/charts/CircleChart.vue";
import Button2 from "@/components/ui/Button2.vue";
import _ from "lodash";
import { mapState } from "vuex";
import {
  Resource,
  ResourceStatus,
  ResourceHealth,
} from "@/api/sdf/model/resource";
import { Entity } from "@/api/sdf/model/entity";

export default Vue.extend({
  name: "ResourcesVisualization",
  components: {
    CircleChart,
    Tooltip,
    Button2,
  },
  props: {
    applicationId: String,
  },
  data() {
    return {
      serviceColor: "78,141,171",
      updating: false,
    };
  },
  methods: {
    statusStyle(resource: Resource): string {
      let createdColor = "0,176,90,1"; // (green)
      let failedColor = "116,42,42,1"; // (dark red)
      let pendingColor = "160,174,192,1"; // (gray)
      let deletedColor = "26,32,44,1"; // (black)
      let programmersSuckColor = "254,178,227,1"; // (yucky pink)

      let color;
      if (resource.status == ResourceStatus.Created) {
        color = createdColor;
      } else if (resource.status == ResourceStatus.Failed) {
        color = failedColor;
      } else if (resource.status == ResourceStatus.Pending) {
        color = pendingColor;
      } else if (resource.status == ResourceStatus.Deleted) {
        color = deletedColor;
      } else {
        // What about in progress colors? lets stay pink! yummy
        color = programmersSuckColor;
      }
      return `color:rgba(${color})`;
    },
    healthStyle(resource: Resource): string {
      let okColor = "0,176,90,1"; // (green)
      let warningColor = "168,107,2,1"; // (green)
      let errorColor = "116,42,42,1"; // (dark red)
      let unknownColor = "160,174,192,1"; // (gray)
      let programmersSuckColor = "254,178,227,1"; // (yucky pink)

      let color;
      if (resource.health == ResourceHealth.Ok) {
        color = okColor;
      } else if (resource.health == ResourceHealth.Warning) {
        color = warningColor;
      } else if (resource.health == ResourceHealth.Error) {
        color = errorColor;
      } else if (resource.health == ResourceHealth.Unknown) {
        color = unknownColor;
      } else {
        console.log("resource health", { resource: resource });
        color = programmersSuckColor;
      }
      return `color:rgba(${color})`;
    },
    resourceName(resource: Resource): string {
      let entity: Entity | undefined = this.$store.state.editor.objects[
        resource.nodeId
      ];
      if (entity) {
        return entity.name;
      } else {
        return "";
      }
    },
    resourceKind(resource: Resource): string {
      let entity: Entity | undefined = this.$store.state.editor.objects[
        resource.nodeId
      ];
      if (entity) {
        return entity.objectType;
      } else {
        return "";
      }
    },
    async sync(): Promise<void> {
      this.updating = true;
      await this.$store.dispatch("editor/syncResources");
      this.updating = false;
    },
  },
  computed: {
    ...mapState({
      resources: (state: any): Resource[] => state.editor.resources,
    }),
    statusColor(): string {
      let stateSuccessColor = "0,179,79";
      let stateFailureColor = "187,107,0";
      let colors = [stateSuccessColor, stateFailureColor];
      return colors[Math.floor(Math.random() * 2)];
    },
  },
});
</script>
