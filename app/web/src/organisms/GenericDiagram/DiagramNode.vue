<template>
  <v-group
    ref="groupRef"
    :config="{
      id: `node-${node.id}`,
      x: position.x,
      y: position.y,
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
  >
    <!-- selection box outline -->
    <v-rect
      v-if="isHovered || isSelected"
      :config="{
        width: nodeWidth + 8,
        height: nodeHeight + 8,
        x: -halfWidth - 4,
        y: -4,
        cornerRadius: CORNER_RADIUS + 3,
        stroke: SELECTION_COLOR,
        strokeWidth: isSelected ? 5 : 2,
        listening: false,
      }"
    />
    <!-- box background - also used by layout manager to figure out nodes location and size -->
    <v-rect
      :config="{
        id: `node-${node.id}--bg`,
        width: nodeWidth,
        height: nodeHeight,
        x: -halfWidth,
        y: 0,
        cornerRadius: CORNER_RADIUS,
        fill: colors.bodyBg,
        fillAfterStrokeEnabled: true,
        stroke: colors.headerBg,
        strokeWidth: 2,
        shadowColor: 'black',
        shadowBlur: 8,
        shadowOffset: { x: 3, y: 3 },
        shadowOpacity: 0.4,
        shadowEnabled: false,
      }"
    />

    <!-- header background -->
    <v-rect
      :config="{
        cornerRadius: [CORNER_RADIUS, CORNER_RADIUS, 0, 0],
        fill: colors.headerBg,
        x: -halfWidth,
        y: 0,
        width: nodeWidth,
        height: headerTextHeight,
        listening: false,
      }"
    />
    <!-- header text -->
    <v-text
      ref="titleTextRef"
      :config="{
        x: -halfWidth,
        y: 0,
        verticalAlign: 'top',
        align: 'center',
        text: node.title,
        width: nodeWidth,
        padding: 5,
        fill: colors.headerText,
        fontStyle: 'bold',
        fontFamily: DIAGRAM_FONT_FAMILY,
        listening: false,
      }"
    />

    <v-text
      ref="subtitleTextRef"
      :config="{
        x: -halfWidth,
        y: nodeHeaderHeight,
        verticalAlign: 'top',
        align: 'center',
        text: node.subtitle,
        width: nodeWidth,
        padding: 5,
        fill: colors.bodyText,
        fontFamily: DIAGRAM_FONT_FAMILY,
        listening: false,
      }"
    />

    <!-- sockets -->
    <v-group
      :config="{
        x: -halfWidth - 1,
        y: nodeHeaderHeight + SOCKET_MARGIN_TOP + subtitleTextHeight,
      }"
    >
      <DiagramNodeSocket
        v-for="(socket, i) in leftSockets"
        :key="socket.id"
        :socket="socket"
        :y="i * SOCKET_GAP"
        :connected-edges="connectedEdges[socket.id]"
        :draw-edge-state="drawEdgeState"
        :node-width="nodeWidth"
        @hover:start="emit('hover:start', socket.id)"
        @hover:end="emit('hover:end', socket.id)"
      />
    </v-group>

    <v-group
      :config="{
        x: halfWidth + 1,
        y:
          nodeHeaderHeight +
          SOCKET_MARGIN_TOP +
          subtitleTextHeight +
          SOCKET_GAP * leftSockets.length,
      }"
    >
      <DiagramNodeSocket
        v-for="(socket, i) in rightSockets"
        :key="socket.id"
        :socket="socket"
        :y="i * SOCKET_GAP"
        :connected-edges="connectedEdges[socket.id]"
        :draw-edge-state="drawEdgeState"
        :node-width="nodeWidth"
        @hover:start="emit('hover:start', socket.id)"
        @hover:end="emit('hover:end', socket.id)"
      />
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import { computed, nextTick, PropType, ref, watch } from "vue";
import _ from "lodash";
import tinycolor from "tinycolor2";

import {
  DiagramDrawEdgeState,
  DiagramEdgeDef,
  DiagramNodeDef,
} from "./diagram_types";
import DiagramNodeSocket from "./DiagramNodeSocket.vue";
import { KonvaEventObject } from "konva/lib/Node";

import {
  SOCKET_GAP,
  CORNER_RADIUS,
  SOCKET_MARGIN_BOTTOM,
  SOCKET_MARGIN_TOP,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
} from "./diagram_constants";
import { useTheme } from "@/composables/injectTheme";
import { Vector2d } from "konva/lib/types";

const props = defineProps({
  node: {
    type: Object as PropType<DiagramNodeDef>,
    required: true,
  },
  tempPosition: {
    type: Object as PropType<Vector2d>,
  },
  connectedEdges: {
    type: Object as PropType<Record<string, DiagramEdgeDef[]>>,
    default: () => ({}),
  },
  drawEdgeState: {
    type: Object as PropType<DiagramDrawEdgeState>,
    default: () => ({}),
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const emit = defineEmits(["resize", "hover:start", "hover:end"]);

const theme = useTheme();

const titleTextRef = ref();
const subtitleTextRef = ref();
const groupRef = ref();

const leftSockets = computed(() =>
  _.filter(props.node.sockets, (s) => s.nodeSide === "left"),
);
const rightSockets = computed(() =>
  _.filter(props.node.sockets, (s) => s.nodeSide === "right"),
);

const nodeWidth = computed(() => 170);
const halfWidth = computed(() => nodeWidth.value / 2);

const headerTextHeight = ref(20);
const subtitleTextHeight = ref(0);
watch(
  [nodeWidth, () => props.node.title, () => props.node.subtitle],
  () => {
    // we have to let the new header be drawn on the canvas before we can check the height
    nextTick(recalcHeaderHeight);
  },
  { immediate: true },
);
function recalcHeaderHeight() {
  headerTextHeight.value =
    titleTextRef.value?.getNode()?.getSelfRect().height || 20;
  subtitleTextHeight.value = props.node.subtitle
    ? subtitleTextRef.value?.getNode()?.getSelfRect().height
    : 10;
}

const nodeHeaderHeight = computed(() => headerTextHeight.value);
const nodeBodyHeight = computed(() => {
  return (
    subtitleTextHeight.value +
    SOCKET_MARGIN_TOP +
    SOCKET_MARGIN_BOTTOM +
    SOCKET_GAP * (leftSockets.value.length + rightSockets.value.length - 1)
  );
});
const nodeHeight = computed(
  () => nodeHeaderHeight.value + nodeBodyHeight.value,
);

const position = computed(() => props.tempPosition || props.node.position);

watch([nodeWidth, nodeHeight, position], () => {
  // we call on nextTick to let the component actually update itself on the stage first
  // because parent responds to this event by finding shapes on the stage and looking at location/dimensions
  nextTick(() => emit("resize"));
});

const colors = computed(() => {
  const primaryColor = tinycolor(props.node.color || DEFAULT_NODE_COLOR);
  const headerText = primaryColor.isDark() ? "#FFF" : "#000";

  // body bg
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  const bodyBg = tinycolor(bodyBgHsl);

  const bodyText = theme.value === "dark" ? "#FFF" : "#000";
  return {
    headerBg: primaryColor.toRgbString(),
    headerText,
    bodyBg: bodyBg.toRgbString(),
    bodyText,
  };
});

function onMouseOver(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:start");
}
function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}
</script>
