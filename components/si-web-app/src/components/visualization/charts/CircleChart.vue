<template>
  <div class="flex">
    <svg :width="width / 2" :height="height">
      <circle :cx="r" :cy="r" :r="r" :style="statusStyle" />
      <circle :cx="r" :cy="r" :r="r - 3" :style="style" />
    </svg>
    <svg :width="width / 2" :height="height">
      <circle :cx="0" :cy="r" :r="r" :style="healthStyle" />
      <circle :cx="0" :cy="r" :r="r - 3" :style="style" />
    </svg>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import {
  Resource,
  ResourceHealth,
  ResourceStatus,
} from "@/api/sdf/model/resource";

export default Vue.extend({
  name: "CircleChart",
  props: {
    resource: {
      type: Object as () => Resource,
    },
    width: {
      type: Number,
      default: 14,
    },
    height: {
      type: Number,
      default: 14,
    },
    r: {
      type: Number,
      default: 7,
    },
    rgbColor: {
      type: String,
      default: "0,142,210,1",
    },
  },
  computed: {
    style(): string {
      return "fill:rgba(" + this.rgbColor + ")";
    },
    statusStyle(): string {
      let createdColor = "0,176,90,1"; // (green)
      let failedColor = "116,42,42,1"; // (dark red)
      let pendingColor = "160,174,192,1"; // (gray)
      let deletedColor = "26,32,44,1"; // (black)
      let programmersSuckColor = "254,178,227,1"; // (yucky pink)

      let color;
      if (this.resource.status == ResourceStatus.Created) {
        color = createdColor;
      } else if (this.resource.status == ResourceStatus.Failed) {
        color = failedColor;
      } else if (this.resource.status == ResourceStatus.Pending) {
        color = pendingColor;
      } else if (this.resource.status == ResourceStatus.Deleted) {
        color = deletedColor;
      } else {
        console.log("resource status", { resource: this.resource });
        color = programmersSuckColor;
      }
      return `fill:rgba(${color})`;
    },
    healthStyle(): string {
      let okColor = "0,176,90,1"; // (green)
      let warningColor = "168,107,2,1"; // (green)
      let errorColor = "116,42,42,1"; // (dark red)
      let unknownColor = "160,174,192,1"; // (gray)
      let programmersSuckColor = "254,178,227,1"; // (yucky pink)

      let color;
      if (this.resource.health == ResourceHealth.Ok) {
        color = okColor;
      } else if (this.resource.health == ResourceHealth.Warning) {
        color = warningColor;
      } else if (this.resource.health == ResourceHealth.Error) {
        color = errorColor;
      } else if (this.resource.health == ResourceHealth.Unknown) {
        color = unknownColor;
      } else {
        console.log("resource health", { resource: this.resource });
        color = programmersSuckColor;
      }
      return `fill:rgba(${color})`;
    },
  },
});
</script>
