<template>
  <span
    :class="
      clsx(
        'border rounded-sm font-normal',
        tighter ? 'leading-snug tracking-tighter px-3xs' : 'py-3xs px-2xs',
        computedClasses,
      )
    "
    ><slot
  /></span>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import clsx from "clsx";
import { useTheme } from "../utils/theme_tools";
import { Tones } from "../utils/color_utils";

const containerTheme = useTheme();

const props = defineProps({
  tighter: { type: Boolean },
  tone: { type: String as PropType<Tones>, default: "action" },
  mono: { type: Boolean, default: false, required: false },
});

const computedClasses = computed(() => ({
  ...(props.tone && { [`--tone-${props.tone}`]: true }),
  "--within-dark": containerTheme.theme.value === "dark",
  "--within-light": containerTheme.theme.value === "light",
  "font-mono pt-3xs": props.mono,
}));
</script>
