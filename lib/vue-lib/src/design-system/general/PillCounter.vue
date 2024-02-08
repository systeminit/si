<template>
  <div
    v-if="count !== 0 || showIfZero"
    :class="
      clsx(
        'inline-block rounded-2xl px-xs border',
        borderTone ? getToneBorderColorClass(borderTone) : 'border-current',
        getToneTextColorClass(tone),
        size && `text-${size}`,
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
  getToneTextColorClass,
  getToneBorderColorClass,
} from "../utils/color_utils";

const props = defineProps({
  count: Number,
  // TODO: implement color/tone options
  tone: { type: String as PropType<Tones>, default: "shade" },
  size: {
    type: String as PropType<"xs" | "sm" | "md" | "base" | "lg" | "xl" | "2xl">,
  },
  showIfZero: { type: Boolean },
  borderTone: { type: String as PropType<Tones> },
});
</script>
