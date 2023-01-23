<template>
  <v-group v-if="shouldDraw && points && centerPoint">
    <v-line
      :config="{
        visible: isSelected,
        points,
        stroke: SELECTION_COLOR,
        strokeWidth: 7,
        listening: false,
      }"
    />
    <v-line
      :config="{
        id: edge.uniqueKey,
        points,
        stroke: strokeColor,
        strokeWidth: isHovered ? 3 : 2,
        hitStrokeWidth: 8,
        listening: !edge.def.isInvisible,
        opacity: isDeleted ? 0.5 : 1,
        dash: [10, 10],
        dashEnabled: isDeleted,
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
      }"
    >
      <template v-if="isAdded">
        <v-circle
          :config="{
            width: 18,
            height: 18,
            fill: diagramConfig?.toneColors?.success,
          }"
        />
        <DiagramIcon
          icon="plus"
          color="#FFFFFF"
          :config="{
            x: -10,
            y: -10,
            width: 20,
            height: 20,
          }"
        />
      </template>
      <template v-else>
        <DiagramIcon
          icon="x"
          :color="diagramConfig?.toneColors?.destructive"
          :config="{
            x: -13,
            y: -13,
            width: 26,
            height: 26,
          }"
        />
      </template>
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import { computed, PropType } from "vue";
import { colors } from "@/utils/design_token_values";
import { useTheme } from "@/ui-lib/theme_tools";
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

const strokeColor = computed(() => {
  if (isDevMode && props.edge.def.isInvisible) {
    return "rgba(100,50,255,0.1)";
  }

  if (isAdded.value) return colors.success[500];
  if (isDeleted.value) return colors.destructive[500];
  return theme.value === "dark" ? colors.shade[0] : colors.shade[100];
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
