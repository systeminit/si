<template>
  <div
    v-if="!(hideIfZero && count === 0)"
    :class="
      clsx(
        'pillcounter',
        paddingX !== 'none' && `px-${paddingX}`,
        paddingY !== 'none' && `py-${paddingY}`,
        'inline-block rounded text-center',
        'flex flex-row items-center justify-around',
        !noColorStyles &&
          (toneToBg
            ? getToneBgColorClass(tone)
            : [
                getToneTextColorClass(tone),
                'bg-neutral-200 dark:bg-neutral-600',
              ]),
        size && `text-${size}`,
        showHoverInsideTreeNode &&
          'group-hover/tree:text-action-500 dark:group-hover/tree:text-action-300 group-hover/tree:bg-action-100 dark:group-hover/tree:bg-action-800 group-hover/tree:border-action-500 dark:group-hover/tree:border-action-300 border border-transparent',
      )
    "
  >
    <slot />
    {{ count }}
  </div>
</template>

<script setup lang="ts">
import { PropType } from "vue";
import clsx from "clsx";
import {
  Tones,
  getToneTextColorClass,
  getToneBgColorClass,
} from "../utils/color_utils";
import { SpacingSizes } from "../utils/size_utils";

const props = defineProps({
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
  toneToBg: { type: Boolean },
});
</script>
