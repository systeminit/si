<template>
  <div class="absolute flex flex-col gap-2 z-10 left-4 top-4">
    <!-- Outline button - opens the outline panel -->
    <div
      v-tooltip="'Diagram Outline'"
      :class="getButtonClasses(false)"
      @click="emit('toggle-outline')"
    >
      <Icon name="bullet-list-indented" size="full" />
    </div>

    <div
      v-tooltip="'Assets'"
      :class="getButtonClasses(false)"
      @click="emit('toggle-assets')"
    >
      <Icon name="component-plus" size="full" />
    </div>

    <!-- Horizontal rule for visual separation -->
    <div class="w-full h-px bg-neutral-200 dark:bg-neutral-700 my-1"></div>

    <!-- Original diagram controls positioned on the left side -->
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
          'w-9 h-9 rounded border flex flex-col justify-center text-center cursor-pointer select-none text-xs',
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
      :class="getButtonClasses(false)"
      @click="emit('toggle-help')"
    >
      <Icon name="question-circle" size="full" />
    </div>

    <div
      v-tooltip="'Generate Workspace Screenshot'"
      :class="getButtonClasses(false)"
      @click="emit('downloadCanvasScreenshot')"
    >
      <Icon name="download" size="full" />
    </div>

    <div
      v-if="featureFlagsStore.AUTO_LAYOUT_EXPERIMENT"
      v-tooltip="'Auto Layout Diagram (Preview)'"
      :class="getButtonClasses(false)"
      @click="emit('autoLayout')"
    >
      <Icon name="tilde-circle" size="full" />
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
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { useViewsStore } from "@/store/views.store";
import { MIN_ZOOM, MAX_ZOOM } from "../ModelingDiagram/diagram_constants";

const ZOOM_LEVEL_OPTIONS = [25, 50, 100, 150, 200];
const featureFlagsStore = useFeatureFlagsStore();
const viewsStore = useViewsStore();
const views = computed(() => viewsStore.viewList);
const selectedView = computed(() => viewsStore.selectView.name);
const props = defineProps<{
  zoomLevel: number;
}>();

const emit = defineEmits<{
  (e: "zoom-in"): void;
  (e: "zoom-out"): void;
  (e: "zoom-reset"): void;
  (e: "toggle-help"): void;
  (e: "toggle-views"): void;
  (e: "toggle-assets"): void;
  (e: "toggle-outline"): void;
  (e: "downloadCanvasScreenshot"): void;
  (e: "autoLayout"): void;
  (e: "set-zoom", level: number): void;
}>();

const zoomMenuRef = ref<InstanceType<typeof DropdownMenu>>();
const viewMenuRef = ref<InstanceType<typeof DropdownMenu>>();
function adjustZoom(direction: "up" | "down") {
  if (direction === "up") {
    emit("zoom-in");
  } else {
    emit("zoom-out");
  }
}

function setZoomLevel(level: number) {
  emit("set-zoom", level);
}

const roundedZoomPercent = computed(() => Math.round(props.zoomLevel * 100));

function getButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full w-9 h-9 p-1 active:border flex items-center justify-center`,
    themeClasses(
      "bg-neutral-600 text-white active:bg-neutral-200 active:text-black active:border-black",
      "bg-neutral-200 text-black active:bg-neutral-700 active:text-white active:border-white",
    ),
    isDisabled
      ? tw`cursor-not-allowed opacity-50`
      : tw`cursor-pointer hover:scale-110`,
  );
}
</script>
