<!-- eslint-disable vue/no-v-html -->
<!--
Note: the 40x30 is intentional for the aws logo et al
-->

<template>
  <svg
    :width="viewBoxX"
    :height="viewBoxY"
    :viewBox="`0 0 40 30`"
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

export type IconSizes = "sm" | "md";

const viewBoxX = computed(() => {
  switch (props.size) {
    case "sm":
      return 35;
    default:
      return 40;
  }
});

const viewBoxY = computed(() => {
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
</script>
