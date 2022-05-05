<template>
  <svg
    viewBox="0 0 20 21"
    version="1.1"
    xmlns="http://www.w3.org/2000/svg"
    xmlns:xlink="http://www.w3.org/1999/xlink"
  >
    <g id="si-icon" stroke-width="1.5" fill="none" fill-rule="evenodd">
      <g id="si-icon-properties">
        <rect
          id="si-icon-rectangle"
          class="si-icon-border-color"
          x="0.5"
          y="0.973684211"
          width="19"
          height="19"
          rx="2"
        ></rect>
        <text
          id="si-icon-text"
          class="si-icon-text-color"
          font-family="Roboto-Regular, Roboto"
          font-size="14"
          font-weight="normal"
          letter-spacing="0.131250005"
        >
          <tspan x="3.81210937" y="15.2">SI</tspan>
        </text>
      </g>
    </g>
  </svg>
</template>

<script lang="ts">
import { defineComponent } from "vue";

export default defineComponent({
  name: "SysinitIcon",
  props: {
    size: {
      type: Number,
      required: false,
      default: 1,
    },
  },
  data: function () {
    return {
      width: 1.25, //20px
      height: 1.313, // 21px
      resolutionSpace: "em",
    };
  },
  computed: {
    resolution: function () {
      return {
        width: (this.width * this.size).toString() + this.resolutionSpace,
        height: (this.height * this.size).toString() + this.resolutionSpace,
      };
    },
  },
});
</script>

<style lang="css" scoped>
.si-icon-border-color {
  stroke: #d1d1d1;
}

.si-icon-text-color {
  fill: #d1d1d1;
}
</style>
