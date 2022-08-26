/* Small helper on top of KonvaSvgImage that takes an icon name/slug and grabs
it from the diagram config's registry of icons */

<template>
  <KonvaSvgImage
    :raw-svg="rawSvg"
    :color="color"
    :config="config"
    :spin="icon === 'loading' || spin"
  />
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";
import KonvaSvgImage from "./KonvaSvgImage.vue";

const props = defineProps({
  icon: { type: String },
  spin: { type: Boolean },
  color: { type: String },
  config: { type: Object, required: true },
});

const diagramConfig = useDiagramConfig();
const rawSvg = computed(() => {
  if (!props.icon) return;
  return diagramConfig?.value.icons?.[props.icon];
});
</script>
