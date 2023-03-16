<template>
  <v-group
    ref="groupRef"
    :config="{
      id: group.uniqueKey,
      x: position.x,
      y: position.y,
      listening: false,
    }"
  >
    <!--  spinner overlay  -->
    <v-group
      ref="overlay"
      :config="{
        x: -halfWidth,
        y: 0,
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
        :size="overlayIconSize"
        :x="halfWidth"
        :y="nodeBodyHeight / 2"
      />
    </v-group>

    <DiagramIcon
      v-if="isDeleted"
      icon="x"
      :color="diagramConfig?.toneColors?.destructive"
      :size="deletedXSize"
      :x="0"
      :y="nodeHeight / 2"
    />
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
import DiagramNodeSocket from "@/components/GenericDiagram/DiagramNodeSocket.vue";
import {
  SOCKET_GAP,
  SOCKET_MARGIN_TOP,
  CORNER_RADIUS,
  DEFAULT_NODE_COLOR,
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
  GROUP_HEADER_BOTTOM_MARGIN,
  GROUP_TITLE_FONT_SIZE,
  GROUP_RESIZE_HANDLE_SIZE,
} from "@/components/GenericDiagram/diagram_constants";
import {
  DiagramDrawEdgeState,
  DiagramEdgeData,
  DiagramElementUniqueKey,
  DiagramGroupData,
  Size2D,
  SideAndCornerIdentifiers,
  DiagramSocketData,
  ElementHoverMeta,
} from "./diagram_types";

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
  tempSize: {
    type: Object as PropType<Size2D>,
  },
  isHovered: Boolean,
  isSelected: Boolean,
});

const { theme } = useTheme();
const diagramConfig = useDiagramConfig();

const titleTextRef = ref();
const groupRef = ref();

const size = computed(
  () => props.tempSize || props.group.def.size || { width: 500, height: 500 },
);

const nodeWidth = computed(() => size.value.width);
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
const nodeBodyHeight = computed(() => size.value.height);
const nodeHeight = computed(
  () =>
    nodeHeaderHeight.value + GROUP_HEADER_BOTTOM_MARGIN + nodeBodyHeight.value,
);

const position = computed(() => props.tempPosition || props.group.def.position);

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

const isDeleted = computed(() => props.group?.def.changeStatus === "deleted");
const deletedXSize = computed(() =>
  Math.min(nodeHeight.value, nodeWidth.value),
);

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
</script>
