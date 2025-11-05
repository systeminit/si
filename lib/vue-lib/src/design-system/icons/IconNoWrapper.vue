<!-- eslint-disable vue/no-v-html -->
<template>
  <svg
    :width="forcedSizeNumbers ? 30 : widthFromSize"
    :height="forcedSizeNumbers ? 30 : heightFromSize"
    :viewBox="viewBox"
    :fill="name === 'logo-si' ? fillColor : 'none'"
    xmlns="http://www.w3.org/2000/svg"
    v-html="iconSvgRaw"
  ></svg>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import { getToneColorHex, Tones } from "../utils/color_utils";
import { getIconByName, IconNames, IconSizeNumbers } from "./icon_set";

export type IconSizes = "sm" | "md";

const widthFromSize = computed(() => {
  switch (props.size) {
    case "sm":
      return 35;
    default:
      return 40;
  }
});

const heightFromSize = computed(() => {
  switch (props.size) {
    case "sm":
      return 20;
    default:
      return 30;
  }
});

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
  forcedSizeNumbers: {
    type: Object as PropType<IconSizeNumbers>,
  },
  fillColor: {
    type: String,
  },
});

const iconSvgRaw = computed(() => {
  const raw = getIconByName(props.name);
  const updated = raw?.replace(
    /(fill|stroke)="(#[A-F0-9]{3,6}|currentColor|black)"/gi,
    `$1="${toneColor.value ?? "white"}"`,
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

const viewBox = computed(() => {
  if (props.forcedSizeNumbers) {
    return `0 0 ${props.forcedSizeNumbers.viewBoxX} ${props.forcedSizeNumbers.viewBoxY}`;
  } else {
    // TODO(Wendy) - this default viewbox may need to be adjusted
    return `0 0 40 30`;
  }
});
</script>
