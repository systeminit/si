/* Small helper on top of KonvaSvgImage that takes an icon name/slug and grabs
it from the diagram config's registry of icons */

<template>
  <v-group :config="{ x, y, offset }">
    <v-circle
      v-if="circleBg"
      :config="{
        width,
        height,
        fill: bgColor,
        offsetX: -width / 2,
        offsetY: -height / 2,
      }"
    />
    <!-- solid black/white bg hidden behind so that a cut out icon (like a check) wont show through  -->
    <v-rect
      v-if="shadeBg"
      :config="{
        width: size * 0.6,
        height: size * 0.6,
        x: size * 0.2,
        y: size * 0.2,
        fill:
          theme === 'dark' ? COLOR_PALETTE.shade[100] : COLOR_PALETTE.shade[0],
      }"
    />
    <KonvaSvgImage
      :rawSvg="rawSvg"
      :color="color"
      :config="{ width, height }"
      :spin="icon === 'loader' || spin"
    />
  </v-group>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import {
  getIconByName,
  COLOR_PALETTE,
  useTheme,
} from "@si/vue-lib/design-system";
import KonvaSvgImage from "./KonvaSvgImage.vue";

const props = defineProps({
  // ideally we'd add the IconNames type
  // but we allow extra icons to be registered for the diagram so we can't
  icon: { type: String, required: true },
  size: { type: Number, required: true },
  x: { type: Number, default: 0 },
  y: { type: Number, default: 0 },
  color: { type: String, default: "#000000" },
  spin: { type: Boolean },
  bgColor: { type: String },
  shadeBg: { type: Boolean },
  circleBg: { type: Boolean },
  config: { type: Object },
  origin: {
    type: String as PropType<
      "center" | "top-left" | "top-right" | "bottom-left" | "bottom-right"
    >,
    default: "center",
  },
});

const { theme } = useTheme();

const width = computed(() => props.size);
const height = computed(() => props.size);

const offset = computed(() => {
  if (props.origin === "center") {
    return { x: width.value / 2, y: height.value / 2 };
  }
  return {
    x: props.origin.endsWith("-left") ? 0 : width.value,
    y: props.origin.startsWith("top-") ? 0 : height.value,
  };
});

const rawSvg = computed(() => {
  if (!props.icon) return;
  return getIconByName(props.icon);
});
</script>
