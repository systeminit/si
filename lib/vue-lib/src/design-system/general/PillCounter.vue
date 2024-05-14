<template>
  <div
    v-if="!(hideIfZero && count === 0)"
    :class="
      clsx(
        paddingX !== 'none' && `px-${paddingX}`,
        paddingY !== 'none' && `py-${paddingY}`,
        'inline-block',
        altStyle
          ? 'border rounded-2xl'
          : ['rounded', !noColorStyles && 'bg-neutral-200 dark:bg-neutral-600'],
        altStyle && getToneBorderColorClass(tone),
        !noColorStyles && getToneTextColorClass(tone),
        size && `text-${size}`,
        showHoverInsideTreeNode &&
          'group-hover/tree:text-action-500 dark:group-hover/tree:text-action-300 group-hover/tree:bg-action-100 dark:group-hover/tree:bg-action-800 group-hover/tree:border-action-500 dark:group-hover/tree:border-action-300 border border-transparent',
      )
    "
  >
    {{ count }}
  </div>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import clsx from "clsx";
import {
  Tones,
  getToneBorderColorClass,
  getToneTextColorClass,
} from "../utils/color_utils";
import { SpacingSizes } from "../utils/size_utils";

defineProps({
  count: Number,
  // TODO: implement color/tone options
  tone: { type: String as PropType<Tones>, default: "shade" },
  noColorStyles: { type: Boolean },
  size: {
    type: String as PropType<SpacingSizes>,
  },
  hideIfZero: { type: Boolean },
  showHoverInsideTreeNode: { type: Boolean },
  paddingX: { type: String as PropType<SpacingSizes>, default: "2xs" },
  paddingY: { type: String as PropType<SpacingSizes>, default: "none" },

  // an alternative style for PillCounter that is more rounded, removes the background, and keeps a persistent border
  altStyle: { type: Boolean },
});
</script>
