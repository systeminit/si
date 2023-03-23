<template>
  <v-layer>
    <!-- simple background color rect -->
    <v-rect
      :config="{
        x: gridMinX,
        y: gridMinY,
        width: gridWidth,
        height: gridHeight,
        fill: bgColor,
        listening: false,
      }"
    />

    <template v-if="zoomLevel >= 0.25">
      <v-line
        v-for="(y, i) in ySteps"
        :key="`h-gridline-${i}`"
        :config="{
          points: [gridMinX, y, gridMaxX, y],
          stroke: lineColor,
          strokeWidth: lineWidth,
          listening: false,
        }"
      />
      <v-line
        v-for="(x, i) in xSteps"
        :key="`v-gridline-${i}`"
        :config="{
          points: [x, gridMinY, x, gridMaxY],
          stroke: lineColor,
          strokeWidth: lineWidth,
          listening: false,
        }"
      />
    </template>
  </v-layer>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { COLOR_PALETTE, useTheme } from "@si/vue-lib/design-system";

const props = defineProps({
  gridMinX: { type: Number, required: true },
  gridMaxX: { type: Number, required: true },
  gridMinY: { type: Number, required: true },
  gridMaxY: { type: Number, required: true },
  zoomLevel: { type: Number, required: true },
});

const gridWidth = computed(() => props.gridMaxX - props.gridMinX);
const gridHeight = computed(() => props.gridMaxY - props.gridMinY);

// TODO: make dynamic based on zoomLevel
const gridSpacing = computed(() => 20);

// normalizing the grid line width to 1 px regardless of zoomLevel level
const lineWidth = computed(() => 1 / props.zoomLevel);

function generateGridPositions(min: number, max: number) {
  const vals = [] as number[];

  const firstStep = gridSpacing.value * Math.floor(min / gridSpacing.value);
  for (let v = firstStep; v < max; v += gridSpacing.value) {
    vals.push(v);
  }
  return vals;
}

const xSteps = computed(() =>
  generateGridPositions(props.gridMinX, props.gridMaxX),
);
const ySteps = computed(() =>
  generateGridPositions(props.gridMinY, props.gridMaxY),
);

const { theme } = useTheme();
const bgColor = computed(() =>
  theme.value === "dark"
    ? COLOR_PALETTE.neutral[900]
    : COLOR_PALETTE.neutral[50],
);
const lineColor = computed(() =>
  theme.value === "dark"
    ? COLOR_PALETTE.neutral[700]
    : COLOR_PALETTE.neutral[200],
);
</script>
