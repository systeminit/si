<template>
  <div ref="siCanvas" class="w-full h-full">
    <template v-if="styleMode() == 'grid'">
      <SiGrid
        :width="gridDefaultOptions.width"
        :height="gridDefaultOptions.height"
      />
    </template>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import SiGrid from "@/atoms/SiGrid.vue";

import { CgResolution } from "@/api/sicg";

interface IGridOptionsSpecified {
  default?: never;
  width: string;
  height: string;
  size: number;
  backgroundColor: string;
  gridColor: string;
  gridThickness: number;
}

interface IGridOptionsDefault {
  default: true;
  width?: never;
  height?: never;
  size?: never;
  backgroundColor?: never;
  gridColor?: never;
  gridThickness?: never;
}

type IGridOptions = IGridOptionsSpecified | IGridOptionsDefault;
// look at advanced types in the typescript doc

interface IStytleGrid {
  grid: IGridOptions;
  color?: never;
}

interface IStyleColor {
  grid?: never;
  color: string;
}

type ICanvasStyle = IStytleGrid | IStyleColor;

interface IData {
  gridDefaultOptions: IGridOptions;
}

/**
 * SiCanvas
 * Currently, the canvas has a fixed size. We should implement infinite size at some point.
 * The size of the canvas should be set by the size of the content + a border to fill the viewport.
 * As the content grows, the canvas grows... Nodes should be positioned on the canvas from the center
 * of the canvas. The center of the canvas is the origin (0,0).
 *
 */

export default Vue.extend({
  name: "SiBackground",
  components: {
    SiGrid,
  },
  props: {
    resolution: {
      type: Object as PropType<CgResolution>,
      default: {
        x: 100,
        y: 100,
      },
      required: false,
    },
    canvasStyle: {
      type: Object as PropType<ICanvasStyle>,
      default: function() {
        return { default: true };
      },
      required: false,
    },
  },
  data(): IData {
    return {
      gridDefaultOptions: {
        width: this.resolution.x + "px",
        height: this.resolution.y + "px",
        size: 20,
        backgroundColor: "#1E1E1E",
        gridColor: "#323537",
        gridThickness: 1,
      },
    };
  },
  methods: {
    styleMode(): string {
      // this.gridStyle()
      // Compose default grid value or take value from prop...
      return "grid";
    },

    // gridStyle(): IGridOptions {
    //   if (this.style.grid) {
    //     if (!this.style.grid.default) {
    //       return this.style.grid
    //     }
    //   }
    //   return this.gridDefaultOptions
    // }
  },
});
</script>
