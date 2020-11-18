<template>
  <Tooltip>
    <svg :width="width" :height="height">
      <rect :width="width" :height="height" :style="entityStyle" />
      <rect :width="width / 3" :height="height" :style="statusStyle" />
      <rect
        :width="width / 3"
        :height="height"
        :x="(width / 3) * 2"
        :style="healthStyle"
      />
    </svg>
    <template v-slot:tooltip>
      <div class="flex flex-col text-gray-400">
        <div class="text-sm">
          {{ service.name }}
        </div>
        <div class="flex flex-col ml-2 text-xs">
          <div class="flex">
            <div class="pr-2">
              Target
            </div>
            <div>
              {{ deploymentTarget }}
            </div>
          </div>

          <div class="flex">
            <div class="pr-2">
              Status
            </div>
            <div :style="statusTooltip">
              {{ resource ? resource.status : 'unknown' }}
            </div>
          </div>
          <div class="flex">
            <div class="pr-2">
              Health
            </div>
            <div :style="healthTooltip">
              {{ resource ? resource.health : 'unknown' }}
            </div>
          </div>
          <div class="flex">
            <div class="pr-2">
              Time
            </div>
            <div>
              {{ resource ? resource.timestamp : 'unknown' }}
            </div>
          </div>
        </div>
      </div>
    </template>
  </Tooltip>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import _ from "lodash";

import { Entity } from "@/api/sdf/model/entity";
import {
  Resource,
  ResourceStatus,
  ResourceHealth,
} from "@/api/sdf/model/resource";
import { RootStore } from "@/store";
import Tooltip from "@/components/ui/Tooltip.vue";

export default Vue.extend({
  name: "SquareChart",
  components: {
    Tooltip,
  },
  props: {
    service: {
      type: Object as () => Entity,
    },
    inEditor: {
      type: String,
    },
    applicationId: {
      type: String,
    },
    width: {
      type: Number,
      default: 14,
    },
    height: {
      type: Number,
      default: 14,
    },
    rgbColor: {
      type: String,
      default: "0,0,255",
    },
  },
  methods: {
    statusCore(tooltip?: boolean): string {
      let styleName = "fill";
      if (tooltip) {
        styleName = "color";
      }
      let resource = this.resource;
      let createdColor = "0,176,90,1"; // (green)
      let failedColor = "116,42,42,1"; // (dark red)
      let pendingColor = "160,174,192,1"; // (gray)
      let deletedColor = "26,32,44,1"; // (black)
      let programmersSuckColor = "254,178,227,1"; // (yucky pink)

      let color;
      if (!resource) {
        color = programmersSuckColor;
      } else if (resource.status == ResourceStatus.Created) {
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
      return `${styleName}:rgba(${color})`;
    },
    healthCore(tooltip?: boolean): string {
      let styleName = "fill";
      if (tooltip) {
        styleName = "color";
      }
      let resource = this.resource;
      let okColor = "0,176,90,1"; // (green)
      let warningColor = "168,107,2,1"; // (green)
      let errorColor = "116,42,42,1"; // (dark red)
      let unknownColor = "160,174,192,1"; // (gray)
      let programmersSuckColor = "254,178,227,1"; // (yucky pink)

      let color;
      if (!resource) {
        color = programmersSuckColor;
      } else if (resource.health == ResourceHealth.Ok) {
        color = okColor;
      } else if (resource.health == ResourceHealth.Warning) {
        color = warningColor;
      } else if (resource.health == ResourceHealth.Error) {
        color = errorColor;
      } else if (resource.health == ResourceHealth.Unknown) {
        color = unknownColor;
      } else {
        color = programmersSuckColor;
      }
      return `${styleName}:rgba(${color})`;
    },
  },
  computed: {
    resource(): Resource | undefined {
      if (this.inEditor == "true") {
        const result = _.find(this.$store.state.editor.resources, [
          "nodeId",
          this.service.nodeId,
        ]);
        return result;
      } else {
        const result = _.find(
          this.$store.state.application.resources[this.applicationId],
          ["nodeId", this.service.nodeId],
        );
        return result;
      }
    },
    deploymentTarget(): string {
      let deploymentTarget = "unknown";
      if (this.service.properties["__baseline"]) {
        if (this.service.properties["__baseline"].deploymentTarget) {
          deploymentTarget = this.service.properties["__baseline"]
            .deploymentTarget;
        }
      }
      return deploymentTarget;
    },
    entityStyle(): string {
      return "fill:rgba(78,141,171,1)";
    },
    statusStyle(): string {
      return this.statusCore();
    },
    statusTooltip(): string {
      return this.healthCore(true);
    },
    healthStyle(): string {
      return this.healthCore();
    },
    healthTooltip(): string {
      return this.healthCore(true);
    },
  },
});
</script>
