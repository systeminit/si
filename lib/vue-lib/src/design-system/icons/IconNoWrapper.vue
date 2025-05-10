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
  <svg
    width="30"
    height="30"
    viewBox="0 0 30 30"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    v-html="iconSvgRaw"
  ></svg>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import { getToneColorHex, Tones } from "../utils/color_utils";
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
    default: "neutral",
  },
});

const iconSvgRaw = computed(() => {
  const raw = getIconByName(props.name);
  const updated = raw?.replace(
    /(fill|stroke)="(#[A-F0-9]{3,6}|currentColor|black)"/gi,
    `$1="${toneColor.value}"`,
  );
  // eslint-disable-next-line no-useless-escape
  const svgRe = /<svg\b[^>]* (viewBox=\"(\b[^"]*)\").*?>([\s\S]*?)<\/svg>/gim;
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const matches = svgRe.exec(updated!);
  return matches?.[3];
});

const toneColor = computed(() => {
  return props.tone ? getToneColorHex(props.tone) : "currentColor";
});
</script>
