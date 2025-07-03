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
    class="icon shrink-0"
    :class="
      clsx(
        'block rounded-sm',
        sizeClasses,
        toneColorClass,
        computedRotate && `--rotate-${computedRotate}`,
        AUTO_SPIN_ICONS_CLOCKWISE.includes(name) && '--spin',
        AUTO_SPIN_ICONS_COUNTER_CLOCKWISE.includes(name) && '--spin-cc',
        AUTO_SPIN_SLOW_WITH_HORIZONTAL_FLIP_CLOCKWISE.includes(name) &&
          '--spin-slow-with-horizontal-flip',
      )
    "
    v-html="iconSvgRaw"
  />
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import clsx from "clsx";
import { getToneTextColorClass, Tones } from "../utils/color_utils";
import { getIconByName, IconNames } from "./icon_set";

export type IconSizes =
  | "2xs"
  | "xs"
  | "sm"
  | "md"
  | "lg"
  | "xl"
  | "2xl"
  | "full";

const props = defineProps({
  name: { type: String as PropType<IconNames>, required: true },
  rotate: { type: String as PropType<"left" | "right" | "up" | "down"> },
  size: {
    type: String as PropType<IconSizes | "inherit" | "none">,
    default: "md",
  },
  tone: {
    type: String as PropType<Tones>,
  },
});

const iconSvgRaw = computed(() => {
  const raw = getIconByName(props.name);
  return raw?.replace(
    /(fill|stroke)="(#[A-F0-9]{3,6}|currentColor|black)"/gi,
    `$1="currentColor"`,
  );
});

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
      "2xs": "w-3 h-3",
      xs: "w-4 h-4",
      sm: "w-5 h-5",
      md: "w-6 h-6",
      lg: "w-8 h-8",
      xl: "w-9 h-9",
      "2xl": "w-12 h-12",
      inherit: "w-inherit h-inherit",
      none: "",
    }[props.size]),
);

const AUTO_SPIN_ICONS_CLOCKWISE = ["loader"];
const AUTO_SPIN_ICONS_COUNTER_CLOCKWISE = ["refresh-active"];
const AUTO_SPIN_SLOW_WITH_HORIZONTAL_FLIP_CLOCKWISE = ["refresh-carbon-active"];
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
  &.--spin-cc {
    > :deep(svg) {
      @apply animate-spin-cc;
    }
  }
  &.--spin-slow-with-horizontal-flip {
    transform: scaleY(-1);

    > :deep(svg) {
      // Note that we spin counter-clockwise because we've flipped it. That's also why we don't
      // have the flip and the spin separated: it would be confusing to the caller to request
      // counter-clockwise and flip when they truly want clockwise.
      @apply animate-spin-cc;
      animation-duration: 5s;
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
