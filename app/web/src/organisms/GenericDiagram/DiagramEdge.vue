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
      id: edge.uniqueKey,
      points,
      stroke: strokeColor,
      strokeWidth: isHovered ? 3 : 2,
      hitStrokeWidth: 8,
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
    @mousedown="onMouseDown"
  />
</template>

<script lang="ts" setup>
import { KonvaEventObject } from "konva/lib/Node";
import { Vector2d } from "konva/lib/types";
import { computed, PropType } from "vue";
import { colors } from "@/utils/design_token_values";
import { useTheme } from "@/ui-lib/theme_tools";
import { SOCKET_SIZE, SELECTION_COLOR } from "./diagram_constants";
import { DiagramEdgeData } from "./diagram_types";
import { pointAlongLine } from "./utils/math";

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

const { theme } = useTheme();

const strokeColor = computed(() =>
  theme.value === "dark" ? colors.shade[0] : colors.shade[100],
);

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

function onMouseOver(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:start");
}
function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}
function onMouseDown(_e: KonvaEventObject<MouseEvent>) {
  // e.cancelBubble = true; // stops dragging of parent
}

// defineExpose({ recalculatePoints });
</script>
