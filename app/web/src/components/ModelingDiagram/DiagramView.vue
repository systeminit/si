<template>
  <v-group v-if="!occlude">
    <v-rect
      v-if="debug"
      :config="{
        width: renderRect.width,
        height: renderRect.height,
        x: renderRect.x,
        y: renderRect.y,
        fill: 'red',
        opacity: 0.2,
      }"
    />
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
      <v-group ref="cacheRef">
        <v-shape :config="config" />

        <v-text
          v-if="hideDetails !== 'hide'"
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
            wrap: 'word',
          }"
        />
      </v-group>
      <!-- status icons -->
      <v-group
        v-if="statusIcons?.length && hideDetails === 'show'"
        :config="{
          x: (statusIcons.length * 26) / 2,
          y: -20,
        }"
      >
        <template
          v-for="(statusIcon, i) in _.reverse(_.slice(statusIcons))"
          :key="`status-icon-${i}`"
        >
          <v-text
            v-if="hideDetails === 'show'"
            :config="{
              x: i * -26 - 25,
              y: radius - 43,
              align: 'center',
              verticalAlign: 'top',
              width: 25,
              height: 30,
              text: statusIcon.count,
              padding: 2,
              fill: colors.headerText,
              fontSize: 11,
              fontFamily: DIAGRAM_FONT_FAMILY,
              listening: false,
              wrap: 'char',
            }"
          />
          <DiagramIcon
            :icon="statusIcon.icon"
            :color="getToneColorHex(statusIcon.tone)"
            :size="24"
            :x="i * -26"
            :y="radius - 5"
            origin="bottom-right"
          />
        </template>
      </v-group>
      <v-shape v-if="isHovered" :config="selectionConfig" />
    </v-group>
  </v-group>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, reactive, watch, ref, nextTick, onMounted } from "vue";
import { Vector2d, IRect } from "konva/lib/types";
import { KonvaEventObject } from "konva/lib/Node";
import tinycolor from "tinycolor2";
import { useTheme, getToneColorHex, Tones } from "@si/vue-lib/design-system";
import {
  DIAGRAM_FONT_FAMILY,
  SELECTION_COLOR,
  NODE_WIDTH,
  DetailsMode,
} from "@/components/ModelingDiagram/diagram_constants";
import { useViewsStore } from "@/store/views.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { DiagramViewDef, ElementHoverMeta } from "./diagram_types";
import DiagramIcon from "./DiagramIcon.vue";
import { checkRectanglesOverlap } from "./utils/math";
import { useDiagramContext } from "./ModelingDiagram.vue";

const { theme } = useTheme();

const viewsStore = useViewsStore();
const diagramContext = useDiagramContext();

const props = defineProps<{
  view: DiagramViewDef;
  isHovered: boolean;
  isSelected: boolean;
  debug?: boolean;
  hideDetails?: DetailsMode;
  occlusionRect?: IRect;
}>();

const radius = computed(() => {
  if (props.view.width !== props.view.height)
    throw new Error("Width and Height ought to match");
  // protect against trying to render something "too small"
  return Math.max(props.view.width / 2, NODE_WIDTH / 2);
});

// step up & down the font size
const fontSize = computed(() => {
  const parts = props.view.name.split(" ");
  const totalLen = props.view.name.length;
  const maxLen = Math.max(...parts.map((t) => t.length));
  const ratio = props.view.width / maxLen;
  // explicit catch for "many small words"
  if (ratio > 40 && totalLen > 20) {
    return 28;
  }
  return Math.max(14, Math.min(ratio, 40));
});

const colors = computed(() => {
  const primaryColor = tinycolor(props.view.color);
  const bodyBgHsl = primaryColor.toHsl();
  bodyBgHsl.l = theme.value === "dark" ? 0.08 : 0.95;
  const bodyBg = tinycolor(bodyBgHsl);
  let headerText;
  if (bodyBg.toHsl().l < 0.5) {
    headerText = "#FFF";
  } else {
    headerText = "#000";
  }

  return {
    headerBg: primaryColor.toRgbString(),
    bodyBg: bodyBg.toRgbString(),
    headerText,
  };
});

interface ViewStats {
  count: number;
  icon: string;
  tone: Tones;
}

const statusIcons = computed(() => {
  const icons: ViewStats[] = [];
  const stats = viewsStore.viewStats[props.view.id];
  if (!stats) return icons;

  if (stats.components > 0)
    icons.push({
      count: stats.components,
      icon: "check-hex-outline",
      tone: "success",
    });
  if (stats.resources > 0)
    icons.push({ count: stats.resources, icon: "check-hex", tone: "success" });
  if (stats.failed > 0)
    icons.push({
      count: stats.failed,
      icon: "x-hex-outline",
      tone: "destructive",
    });
  return icons;
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
    fill: colors.value.bodyBg,
    stroke: colors.value.headerBg,
    strokeWidth: 2,
    hitStrokeWidth: 0,
    rotation: 90,
  };
});

function onMouseOver(evt: KonvaEventObject<MouseEvent>, type?: string) {
  evt.cancelBubble = true;
  viewsStore.setHoveredComponentId(
    props.view.id,
    type ? ({ type } as ElementHoverMeta) : undefined,
  );
}

function onMouseOut() {
  viewsStore.setHoveredComponentId(null);
}

const goto = (e: KonvaEventObject<MouseEvent>) => {
  // only allow double click with left mouse button!
  if (e.evt.button === 0) {
    viewsStore.selectView(props.view.id);
  }
};

const RENDER_RECT_EDGE = 8;

const renderRect = computed(() => {
  const size = radius.value + RENDER_RECT_EDGE;
  return {
    x: props.view.x - size,
    y: props.view.y - size,
    width: size * 2,
    height: size * 2,
  };
});

const occlude = computed(() => {
  if (!props.occlusionRect) {
    return false;
  } else {
    const o = !checkRectanglesOverlap(renderRect.value, props.occlusionRect);
    return o;
  }
});

const featureFlagsStore = useFeatureFlagsStore();
const cacheRef = ref();

const cache = () => {
  if (!featureFlagsStore.DIAGRAM_OPTIMIZATION_2 || occlude.value) return;

  // this allows us to fire the cache via the watcher on occlusion
  nextTick(() => {
    const node = cacheRef.value?.getNode();
    if (node) {
      node.cache({
        width: radius.value * 2,
        height: radius.value * 2,
        x: -radius.value,
        y: -radius.value,
        pixelRatio: diagramContext.zoomLevel.value,
      });
    }
  });
};

const clearCache = () => {
  const node = cacheRef.value?.getNode();
  if (node) {
    node.clearCache();
  }
};

onMounted(() => {
  cache();
});

// cache nodes when they come on screen
watch(
  () => occlude.value,
  () => {
    cache();
  },
);

// re-run caching when switching between semantic zoom states
watch(
  () => props.hideDetails,
  () => {
    cache();
  },
);

// re-run caching if the view name changes
watch(
  () => props.view.name,
  () => {
    cache();
  },
);

// re-run caching when zoomed in close enough
watch(
  () => diagramContext.zoomLevel.value,
  (zoomLevel) => {
    if (zoomLevel > 1) {
      cache();
    }
  },
  { deep: true },
);

defineExpose({ clearCache });
</script>
