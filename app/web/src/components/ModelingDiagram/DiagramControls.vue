<template>
  <div :class="clsx('absolute flex left-4 bottom-4 z-20 h-8')">
    <div
      v-tooltip="'Zoom Out'"
      :class="getButtonClasses(zoomLevel <= MIN_ZOOM)"
      @click="adjustZoom('down')"
    >
      <Icon name="minus" size="full" />
    </div>

    <div
      :class="
        clsx(
          themeClasses(
            'bg-white border-neutral-300 text-black hover:border-black',
            'bg-black text-white border-black hover:border-white',
          ),
          'w-20 mx-2  rounded border flex flex-col justify-center text-center cursor-pointer select-none',
        )
      "
      title="set zoom level"
      @click="zoomMenuRef?.open"
    >
      {{ roundedZoomPercent }}%

      <DropdownMenu ref="zoomMenuRef" forceAbove>
        <DropdownMenuItem
          v-for="zoomOptionAmount in ZOOM_LEVEL_OPTIONS"
          :key="zoomOptionAmount"
          class="justify-end"
          @select="setZoomLevel(zoomOptionAmount / 100)"
        >
          {{ zoomOptionAmount }}%
        </DropdownMenuItem>
      </DropdownMenu>
    </div>

    <div
      v-tooltip="'Zoom In'"
      :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
      @click="adjustZoom('up')"
    >
      <Icon name="plus" size="full" />
    </div>

    <div
      v-tooltip="'Diagram Controls'"
      class="ml-4"
      :class="getButtonClasses(false)"
      @click="emit('open:help')"
    >
      <Icon name="question-circle" size="full" />
    </div>

    <div
      v-tooltip="displayModeTooltip"
      class="ml-4"
      :class="
        edgeDisplayMode === 'EDGES_OVER'
          ? getButtonClasses(false)
          : getInvertedButtonClasses(false)
      "
      @click="toggleEdgeDisplayMode"
    >
      <Icon name="eye" size="full" />
    </div>

    <div
      v-tooltip="'Generate Workspace Screenshot'"
      class="ml-4"
      :class="getButtonClasses(false)"
      @click="emit('downloadCanvasScreenshot')"
    >
      <Icon name="download" size="full" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref } from "vue";
import * as _ from "lodash-es";
import clsx from "clsx";
import { tw } from "@si/vue-lib";
import {
  DropdownMenu,
  DropdownMenuItem,
  Icon,
  themeClasses,
} from "@si/vue-lib/design-system";
import { MAX_ZOOM, MIN_ZOOM } from "./diagram_constants";
import { useDiagramContext } from "./ModelingDiagram.vue";

const ZOOM_LEVEL_OPTIONS = [25, 50, 100, 150, 200];

const diagramContext = useDiagramContext();
const { edgeDisplayMode, toggleEdgeDisplayMode, zoomLevel, setZoomLevel } =
  diagramContext;

const displayModeTooltip = computed(() => ({
  content:
    edgeDisplayMode.value === "EDGES_OVER" ? "Edges Over" : "Edges Under",
  hideTriggers: ["hover", "focus", "touch"],
}));

const emit = defineEmits<{
  (e: "open:help"): void;
  (e: "downloadCanvasScreenshot"): void;
}>();

const zoomMenuRef = ref<InstanceType<typeof DropdownMenu>>();

function adjustZoom(direction: "up" | "down") {
  const mult = direction === "down" ? -1 : 1;
  setZoomLevel(zoomLevel.value + (10 / 100) * mult);
}

const roundedZoomPercent = computed(() => Math.round(zoomLevel.value * 100));

function getButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full p-1 active:border`,
    themeClasses(
      "bg-neutral-600 text-white active:bg-neutral-200 active:text-black active:border-black",
      "bg-neutral-200 text-black active:bg-neutral-700 active:text-white active:border-white",
    ),
    isDisabled
      ? tw`cursor-not-allowed opacity-50`
      : tw`cursor-pointer hover:scale-110`,
  );
}

function getInvertedButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full p-1 border`,
    themeClasses(
      "bg-neutral-200 text-black border-black",
      "bg-neutral-700 text-white border-white",
    ),
    isDisabled
      ? tw`cursor-not-allowed opacity-50`
      : tw`cursor-pointer hover:scale-110`,
  );
}
</script>
