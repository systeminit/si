<template>
  <v-group v-if="shouldDraw && points && centerPoint">
    <v-line
      :config="{
        visible: isHovered || isSelected,
        points,
        stroke: SELECTION_COLOR,
        strokeWidth: isSelected ? 9 : 5,
        listening: false,
      }"
    />
    <!-- <v-line
      :config="{
        points,
        stroke: '#000',
        strokeWidth: 4,
        listening: false,
        opacity: 0.4,
      }"
      @mouseover="onMouseOver"
      @mouseout="onMouseOut"
      @mousedown="onMouseDown"
    /> -->
    <v-line
      :config="{
        id: edge.uniqueKey,
        points,
        stroke: strokeColor,
        strokeWidth: 2,
        hitStrokeWidth: 10,
        listening: !edge.def.isInvisible,
        opacity: isDeleted ? 0.5 : 1,
        dash: [10, 10],
        dashEnabled: isDeleted,
        shadowColor: '#000',
        shadowBlur: 1,
        shadowEnabled: isHovered || isSelected,
      }"
      @mouseover="onMouseOver"
      @mouseout="onMouseOut"
      @mousedown="onMouseDown"
    />

    <v-group
      v-if="isAdded || isDeleted"
      :config="{
        x: centerPoint.x,
        y: centerPoint.y,
        listening: false,
      }"
    >
      <template v-if="isAdded">
        <DiagramIcon
          icon="plus"
          :color="
            theme === 'dark' ? COLOR_PALETTE.shade[100] : COLOR_PALETTE.shade[0]
          "
          circleBg
          :bgColor="diagramConfig?.toneColors?.success"
          :size="20"
        />
      </template>
      <template v-else>
        <DiagramIcon
          icon="x"
          :color="diagramConfig?.toneColors?.destructive"
          :size="26"
        />
      </template>
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import { computed, PropType } from "vue";
import { COLOR_PALETTE, useTheme } from "@si/vue-lib/design-system";
import { SOCKET_SIZE, SELECTION_COLOR } from "./diagram_constants";
import { DiagramEdgeData } from "./diagram_types";
import { pointAlongLinePct, pointAlongLinePx } from "./utils/math";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";

const isDevMode = import.meta.env.DEV;

const props = defineProps({
  edge: {
    type: Object as PropType<DiagramEdgeData>,
    required: true,
  },
  fromPoint: {
    type: Object as PropType<Vector2d>,
    default: undefined,
  },
  toPoint: {
    type: Object as PropType<Vector2d>,
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const emit = defineEmits(["hover:start", "hover:end"]);

const diagramConfig = useDiagramConfig();

const { theme } = useTheme();

const isDeleted = computed(() => props.edge.def.changeStatus === "deleted");
const isAdded = computed(() => props.edge.def.changeStatus === "added");

const defaultStrokeColor = computed(() =>
  theme.value === "dark" ? COLOR_PALETTE.shade[0] : COLOR_PALETTE.shade[100],
);

const strokeColor = computed(() => {
  if (isDevMode && props.edge.def.isInvisible) {
    return "rgba(100,50,255,0.1)";
  }

  if (isAdded.value) return COLOR_PALETTE.success[500];
  if (isDeleted.value) return COLOR_PALETTE.destructive[500];
  return defaultStrokeColor.value;
});

const points = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  const fromPointWithGap = pointAlongLinePx(
    props.fromPoint,
    props.toPoint,
    SOCKET_SIZE / 2,
  );
  const toPointWithGap = pointAlongLinePx(
    props.toPoint,
    props.fromPoint,
    SOCKET_SIZE / 2,
  );
  return [
    fromPointWithGap.x,
    fromPointWithGap.y,
    toPointWithGap.x,
    toPointWithGap.y,
  ];
});

const centerPoint = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  return pointAlongLinePct(props.fromPoint, props.toPoint, 0.5);
});

function onMouseOver(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:start");
}

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}

function onMouseDown(_e: KonvaEventObject<MouseEvent>) {
  // e.cancelBubble = true; // stops dragging of parent
}

const shouldDraw = computed(() =>
  isDevMode ? true : !props.edge.def.isInvisible,
);

// defineExpose({ recalculatePoints });
</script>
