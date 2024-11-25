<template>
  <v-group
    ref="groupRef"
    :config="{
      id: view.id,
      x: view.x,
      y: view.y,
    }"
    @mouseover="onMouseOver"
    @mouseout="onMouseOut"
    @dblclick="goto"
  >
    <v-shape :config="config" />

    <v-text
      :config="{
        x: -radius * 0.8,
        y: -radius,
        align: 'center',
        verticalAlign: 'middle',
        width: view.width * 0.8,
        height: view.width,
        text: view.name,
        padding: 2,
        fill: colors.headerText,
        fontSize: fontSize,
        fontFamily: DIAGRAM_FONT_FAMILY,
        listening: false,
        wrap: 'char',
      }"
    />

    <v-shape v-if="isHovered" :config="selectionConfig" />
  </v-group>
</template>

<script lang="ts" setup>
import { computed, reactive, watch } from "vue";
import { Vector2d } from "konva/lib/types";
import { KonvaEventObject } from "konva/lib/Node";
import {
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
} from "@/components/ModelingDiagram/diagram_constants";
import { useViewsStore } from "@/store/views.store";
import { DiagramViewDef, ElementHoverMeta } from "./diagram_types";

const viewStore = useViewsStore();

const props = defineProps<{
  view: DiagramViewDef;
  isHovered: boolean;
  isSelected: boolean;
}>();

const radius = computed(() => {
  if (props.view.width !== props.view.height)
    throw new Error("Width and Height ought to match");
  return props.view.width / 2;
});

const colors = computed(() => {
  return {
    headerText: "black",
  };
});

// step up & down the font size
const fontSize = computed(() => {
  switch (true) {
    case props.view.width >= 500:
      return 81;
    case props.view.width >= 300:
      return 64;
    case props.view.width >= 200:
      return 32;
    default:
      return 20;
  }
});

const points = reactive<Vector2d[]>([]);
const selectionPoints = reactive<Vector2d[]>([]);

const SELECTION_PADDING = 4;
watch(
  () => props.view.width,
  () => {
    const sides = 6; // hexagon
    points.splice(0, points.length);
    const increment = (2 * Math.PI) / sides;
    let angle = 0;

    // find each point of the hex from the center
    for (let i = 0; i < sides; i++) {
      points.push({
        x: radius.value * Math.cos(angle),
        y: radius.value * Math.sin(angle),
      });

      selectionPoints.push({
        x: (radius.value + SELECTION_PADDING) * Math.cos(angle),
        y: (radius.value + SELECTION_PADDING) * Math.sin(angle),
      });

      angle += increment;
    }
  },
  { immediate: true },
);

const selectionConfig = computed(() => {
  return {
    width: props.view.width,
    height: props.view.height,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    sceneFunc(context: any, shape: any) {
      context.beginPath();
      selectionPoints.forEach((pt, idx) => {
        if (idx === 0) context.moveTo(pt.x, pt.y);
        else context.lineTo(pt.x, pt.y);
      });
      context.closePath();

      context.fillStrokeShape(shape);
    },
    stroke: SELECTION_COLOR,
    strokeWidth: 1,
    hitStrokeWidth: 0,
    rotation: 90,
  };
});

// XY represents the center, not the top left or center top
const config = computed(() => {
  return {
    width: props.view.width,
    height: props.view.height,
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    sceneFunc(context: any, shape: any) {
      context.beginPath();
      points.forEach((pt, idx) => {
        if (idx === 0) context.moveTo(pt.x, pt.y);
        else context.lineTo(pt.x, pt.y);
      });
      context.closePath();

      context.fillStrokeShape(shape);
    },
    fill: props.view.color,
    stroke: "black",
    strokeWidth: 2,
    hitStrokeWidth: 0,
    rotation: 90,
  };
});

function onMouseOver(evt: KonvaEventObject<MouseEvent>, type?: string) {
  evt.cancelBubble = true;
  viewStore.setHoveredComponentId(
    props.view.id,
    type ? ({ type } as ElementHoverMeta) : undefined,
  );
}

function onMouseOut() {
  viewStore.setHoveredComponentId(null);
}

const goto = () => {
  viewStore.selectView(props.view.id);
};
</script>
