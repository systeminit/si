<template>
  <v-line
    v-if="points"
    :config="{
      visible: isSelected,
      points,
      stroke: SELECTION_COLOR,
      strokeWidth: 7,
      listening: false,
    }"
  />
  <v-line
    v-if="points"
    :config="{
      id: `edge-${edge.id}`,
      points,
      stroke: strokeColor,
      strokeWidth: isHovered ? 3 : 2,
      hitStrokeWidth: 8,
      // fill: '#A752DE',
      // pointerAtBeginning: edge.isBidirectional,
      // pointerAtEnding: true,
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
    @mousedown="onMouseDown"
  />
</template>

<script lang="ts" setup>
import { useTheme } from "@/composables/injectTheme";
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import { computed, nextTick, onMounted, PropType, ref, watch } from "vue";
import { SOCKET_SIZE, SELECTION_COLOR } from "./diagram_constants";
import { DiagramEdgeDef, DiagramNodeDef } from "./diagram_types";
import { pointAlongLine } from "./utils/math";

const props = defineProps({
  edge: {
    type: Object as PropType<DiagramEdgeDef>,
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

const theme = useTheme();

const strokeColor = computed(() => (theme.value === "dark" ? "#FFF" : "#000"));

const points = computed(() => {
  if (!props.fromPoint || !props.toPoint) return;
  const fromPointWithGap = pointAlongLine(
    props.fromPoint,
    props.toPoint,
    SOCKET_SIZE / 2,
  );
  const toPointWithGap = pointAlongLine(
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

function onMouseOver(e: KonvaEventObject<MouseEvent>) {
  emit("hover:start");
}
function onMouseOut(e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}
function onMouseDown(e: KonvaEventObject<MouseEvent>) {
  // e.cancelBubble = true; // stops dragging of parent
}

// defineExpose({ recalculatePoints });
</script>
