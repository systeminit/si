<template>
  <svg
    ref="grid"
    id="node-grid"
    :width="width"
    :height="height"
    xmlns="http://www.w3.org/2000/svg"
  >
    <defs>
      <pattern
        id="grid"
        :width="size"
        :height="size"
        patternUnits="userSpaceOnUse"
      >
        <rect :fill="backgroundColor" :width="size" :height="size" />
        <path
          :d="gridShape()"
          fill="none"
          :stroke="gridColor"
          :stroke-width="gridThickness"
        />
      </pattern>
    </defs>
    <rect width="100%" height="100%" fill="url(#grid)" />
  </svg>
</template>

<script lang="ts">
import Vue from "vue";

export default Vue.extend({
  name: "SiGrid",
  props: {
    width: {
      type: String,
      default: "100%",
      required: false,
    },
    height: {
      type: String,
      default: "100%",
      required: false,
    },
    size: {
      type: Number,
      default: 20,
      required: false,
    },
    backgroundColor: {
      type: String,
      default: "#1E1E1E",
      required: false,
    },
    gridColor: {
      type: String,
      default: "#323537",
      required: false,
    },
    gridThickness: {
      type: Number,
      default: 1,
      required: false,
    },
  },
  methods: {
    gridShape(): string {
      return `M ${this.size} 0 L 0 0 0 ${this.size}`;
    },
  },
});
</script>
