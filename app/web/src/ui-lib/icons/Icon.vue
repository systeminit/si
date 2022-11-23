<!-- eslint-disable vue/no-v-html -->
<!--
  General Icon component to use throughout the codebase

  Why not just import the icons directly?
  - single import rather than importing many icons in each file, no need to change import to try different icon
  - easier to keep icons consistent and swap all icons of a certain type at once (ex: use the same "x-circle" everywhere)
  - allows multiple aliases for the same icon so the use can be a bit more specific (ex: "qualification-passing")
  - easier to apply consistent styling throughout
  - using a simple string lets us easily add `icon` properties on other components (like buttons / form inputs)
  - rotation helpers so we can use a single icon for each direction of things like arrows / carets
-->

<template>
  <div
    class="icon"
    :class="
      clsx(
        'block',
        !allowPointerEvents && 'pointer-events-none',
        sizeClasses,
        toneColorClass,
        computedRotate && `--rotate-${computedRotate}`,
        AUTO_SPIN_ICONS.includes(name) && '--spin',
      )
    "
    v-html="iconSvgRaw"
  />
</template>

<script lang="ts" setup>
import _ from "lodash";
import { computed, PropType } from "vue";
import clsx from "clsx";
import { getToneTextColorClass, Tones } from "@/ui-lib/helpers/tones";
import { getIconByName, IconNames } from "./icon_set";

export type IconSizes = "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "full";

const props = defineProps({
  name: { type: String as PropType<IconNames>, required: true },
  rotate: { type: String as PropType<"left" | "right" | "up" | "down"> },
  size: {
    type: String as PropType<IconSizes>,
    default: "md",
  },
  tone: {
    type: String as PropType<Tones>,
  },
  allowPointerEvents: { type: Boolean },
});

const iconSvgRaw = computed(() => getIconByName(props.name));

const toneColorClass = computed(() => {
  return props.tone ? getToneTextColorClass(props.tone) : undefined;
});

const computedRotate = computed(() => {
  if (props.rotate) return props.rotate;
  if (props.name.includes("--")) return props.name.split("--")[1];
  return null;
});

const sizeClasses = computed(
  () =>
    ({
      full: "w-full h-full",
      xs: "w-4 h-4",
      sm: "w-5 h-5",
      md: "w-6 h-6",
      lg: "w-8 h-8",
      xl: "w-9 h-9",
      "2xl": "w-12 h-12",
    }[props.size]),
);

const AUTO_SPIN_ICONS = ["loader", "refresh-active"];
</script>

<style lang="less" scoped>
// Using style's here rather than tw classes because we can't (easily) add classes on the svg tag rendered via v-html
.icon {
  // NOTE - have to use deep selectors here since the svg is being rendered via v-html
  > :deep(svg) {
    width: 100%;
    height: 100%;

    @apply w-full h-full transition-transform duration-300;
  }
  &.--spin {
    > :deep(svg) {
      @apply animate-spin;
    }
  }
  &.--rotate-up > :deep(svg) {
    transform: rotate(0);
  }
  &.--rotate-right > :deep(svg) {
    transform: rotate(90deg);
  }
  &.--rotate-down > :deep(svg) {
    transform: rotate(180deg);
  }
  &.--rotate-left > :deep(svg) {
    transform: rotate(270deg);
  }
}
</style>
