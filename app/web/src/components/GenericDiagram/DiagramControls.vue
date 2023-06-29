<template>
  <div :class="clsx('absolute flex left-4 bottom-4 z-20 h-8')">
    <div
      :class="getButtonClasses(zoomLevel <= MIN_ZOOM)"
      @click="adjustZoom('down')"
    >
      <Icon name="minus" size="full" />
    </div>

    <div
      :class="
        clsx(
          'bg-white border-neutral-300 border text-black',
          'dark:bg-black dark:text-white dark:border-black',
          'w-20 mx-2 flex flex-col justify-center text-center cursor-pointer select-none',
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
          @select="emit('update:zoom', zoomOptionAmount / 100)"
        >
          {{ zoomOptionAmount }}%
        </DropdownMenuItem>
      </DropdownMenu>
    </div>

    <div
      :class="getButtonClasses(zoomLevel >= MAX_ZOOM)"
      @click="adjustZoom('up')"
    >
      <Icon name="plus" size="full" />
    </div>

    <div
      class="ml-4"
      :class="getButtonClasses(false)"
      @click="emit('open:help')"
    >
      <Icon name="help-circle" size="full" />
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
} from "@si/vue-lib/design-system";
import { MAX_ZOOM, MIN_ZOOM } from "./diagram_constants";

const ZOOM_LEVEL_OPTIONS = [25, 50, 100, 150, 200];

const props = defineProps({
  zoomLevel: { type: Number, required: true },
});

const emit = defineEmits<{
  (e: "update:zoom", newZoom: number): void;
  (e: "open:help"): void;
}>();

const zoomMenuRef = ref<InstanceType<typeof DropdownMenu>>();

function adjustZoom(direction: "up" | "down") {
  const mult = direction === "down" ? -1 : 1;
  emit("update:zoom", props.zoomLevel + (10 / 100) * mult);
}

const roundedZoomPercent = computed(() => Math.round(props.zoomLevel * 100));

function getButtonClasses(isDisabled: boolean) {
  return clsx(
    tw`rounded-full p-1 bg-neutral-600 text-white`,
    tw`dark:bg-gray-200 dark:text-black`,
    isDisabled
      ? tw`cursor-not-allowed opacity-50`
      : tw`cursor-pointer hover:scale-110`,
  );
}
</script>
