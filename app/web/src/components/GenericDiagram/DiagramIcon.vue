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
import { getIconByName } from "@si/vue-lib/design-system";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";
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
  circleBg: { type: Boolean },
  config: { type: Object },
  origin: {
    type: String as PropType<
      "center" | "top-left" | "top-right" | "bottom-left" | "bottom-right"
    >,
    default: "center",
  },
});

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

const diagramConfig = useDiagramConfig();
const rawSvg = computed(() => {
  if (!props.icon) return;
  const iconFromDiagramConfig = diagramConfig?.value.icons?.[props.icon];
  // diagram config specific icons take precedence
  // but then we look in our full icon set
  return iconFromDiagramConfig || getIconByName(props.icon);
});
</script>
