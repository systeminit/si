<template>
  <v-group
    ref="groupRef"
    :config="{
      id: group.uniqueKey,
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
        id: `${group.uniqueKey}--bg`,
        width: nodeWidth,
        height: nodeHeight,
        x: -halfWidth,
        y: 0,
      }"
    />

    <!--  Node Body  -->
    <v-rect
      :config="{
        id: `${group.uniqueKey}--body`,
        width: nodeWidth,
        height: nodeBodyHeight,
        x: -halfWidth,
        y: nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN,
        cornerRadius: CORNER_RADIUS,
        fill: colors.bodyBg,
        fillAfterStrokeEnabled: true,
        stroke: colors.headerBg,
        strokeWidth: 3,
        dash: [8, 8],
        shadowColor: 'black',
        shadowBlur: 8,
        shadowOffset: { x: 3, y: 3 },
        shadowOpacity: 0.4,
        shadowEnabled: false,
      }"
    />

    <!-- header background -->
    <!--  TODO check with mark what this width should be   -->
    <v-rect
      :config="{
        cornerRadius: CORNER_RADIUS,
        fill: colors.headerBg,
        x: -halfWidth,
        y: 0,
        width: headerWidth,
        height: headerTextHeight,
        listening: false,
      }"
    />

    <!-- header text -->
    <!--  TODO fix font size   -->
    <v-text
      ref="titleTextRef"
      :config="{
        x: -halfWidth,
        y: 0,
        verticalAlign: 'top',
        align: 'left',
        width: headerWidth,
        text: `${group.def.subtitle}: 0`,
        padding: 6,
        fill: colors.headerText,
        fontSize: GROUP_TITLE_FONT_SIZE,
        fontStyle: 'bold',
        fontFamily: DIAGRAM_FONT_FAMILY,
        listening: false,
        wrap: 'none',
        ellipsis: true,
      }"
    />

    <!--  spinner overlay  -->
    <v-group
      ref="overlay"
      :config="{
        x: -halfWidth,
        y: nodeHeaderHeight + GROUP_HEADER_BOTTOM_MARGIN,
        opacity: 0,
        listening: false,
      }"
    >
      <!--  transparent overlay  -->
      <v-rect
        :config="{
          width: nodeWidth,
          height: nodeBodyHeight,
          x: 0,
          y: 0,
          cornerRadius: [0, 0, CORNER_RADIUS, CORNER_RADIUS],
          fill: 'rgba(255,255,255,0.70)',
        }"
      />
      <DiagramIcon
        icon="loader"
        :color="diagramConfig?.toneColors?.['info'] || '#AAA'"
        :config="{
          x: halfWidth - overlayIconSize / 2,
          y: nodeBodyHeight / 2 - overlayIconSize / 2,
          width: overlayIconSize,
          height: overlayIconSize,
        }"
      />
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import { computed, nextTick, PropType, ref, watch } from "vue";
import _ from "lodash";
import tinycolor from "tinycolor2";

import { KonvaEventObject } from "konva/lib/Node";
import { Tween } from "konva/lib/Tween";
import { Vector2d } from "konva/lib/types";
import { useTheme } from "@/ui-lib/theme_tools";
import { DiagramDrawEdgeState, DiagramGroupData } from "./diagram_types";

import {
  CORNER_RADIUS,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
  GROUP_HEADER_BOTTOM_MARGIN,
  GROUP_TITLE_FONT_SIZE,
} from "./diagram_constants";
import DiagramIcon from "./DiagramIcon.vue";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";

const props = defineProps({
  group: {
    type: Object as PropType<DiagramGroupData>,
    required: true,
  },
  tempPosition: {
    type: Object as PropType<Vector2d>,
  },
  drawEdgeState: {
    type: Object as PropType<DiagramDrawEdgeState>,
    default: () => ({}),
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const emit = defineEmits(["resize", "hover:start", "hover:end"]);

const { theme } = useTheme();
const diagramConfig = useDiagramConfig();

const titleTextRef = ref();
const groupRef = ref();

// TODO(Paul) recalculate the group width based on the number of components
const nodeWidth = computed(() => 500);
const halfWidth = computed(() => nodeWidth.value / 2);
// TODO(Victor): this is wrong. headerWidth should be the smallest value between the actual text width and nodeWidth
const headerWidth = computed(() => nodeWidth.value * 0.75);

const overlayIconSize = computed(() => nodeWidth.value / 3);

const headerTextHeight = ref(20);
watch(
  [nodeWidth, () => props.group.def.title, () => props.group.def.subtitle],
  () => {
    // we have to let the new header be drawn on the canvas before we can check the height
    nextTick(recalcHeaderHeight);
  },
  { immediate: true },
);

function recalcHeaderHeight() {
  headerTextHeight.value =
    titleTextRef.value?.getNode()?.getSelfRect().height || 20;
}

const nodeHeaderHeight = computed(() => headerTextHeight.value);
// TODO(Paul) calculate the group height based on the number of components it contains
const nodeBodyHeight = computed(() => {
  return 500;
});
const nodeHeight = computed(
  () =>
    nodeHeaderHeight.value + GROUP_HEADER_BOTTOM_MARGIN + nodeBodyHeight.value,
);

const position = computed(() => props.tempPosition || props.group.def.position);

watch([nodeWidth, nodeHeight, position], () => {
  // we call on nextTick to let the component actually update itself on the stage first
  // because parent responds to this event by finding shapes on the stage and looking at location/dimensions
  nextTick(() => emit("resize"));
});

const colors = computed(() => {
  const primaryColor = tinycolor(props.group.def.color || DEFAULT_NODE_COLOR);
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

const overlay = ref();
watch([() => props.group.def.isLoading, overlay], ([isLoading]) => {
  if (_.isNil(overlay)) return;
  const node = overlay.value.getNode();

  const transition = new Tween({
    node,
    duration: 0.1,
    opacity: isLoading ? 1 : 0,
  });

  transition.play();
});

function onMouseOver(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:start");
}

function onMouseOut(_e: KonvaEventObject<MouseEvent>) {
  emit("hover:end");
}
</script>
