/* Small helper on top of KonvaSvgImage that takes an icon name/slug and grabs
it from the diagram config's registry of icons */

<template>
  <KonvaSvgImage
    :raw-svg="rawSvg"
    :color="color"
    :config="config"
    :spin="icon === 'loader' || spin"
  />
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { getIconByName } from "@/ui-lib/icons/icon_set";
import { useDiagramConfig } from "./utils/use-diagram-context-provider";
import KonvaSvgImage from "./KonvaSvgImage.vue";

const props = defineProps({
  // ideally we'd add the IconNames type
  // but we allow extra icons to be registered for the diagram so we can't
  icon: { type: String },
  spin: { type: Boolean },
  color: { type: String },
  config: { type: Object, required: true },
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
