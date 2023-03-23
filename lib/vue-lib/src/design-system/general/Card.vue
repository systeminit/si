<template>
  <div
    :class="
      clsx(
        themeClasses('bg-neutral-200 text-black', 'bg-neutral-800 text-white'),
        'p-lg shadow',
        rounded && 'rounded-sm',
      )
    "
  >
    <slot />

    <hr
      v-if="$slots.footer"
      :class="
        clsx(
          'border -mx-lg my-lg',
          themeClasses('border-neutral-100', 'border-neutral-900'),
        )
      "
    />
    <slot name="footer" />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "../utils/theme_tools";

const props = defineProps({
  rounded: Boolean,
  // TODO: add padding size props (responsive)
  // TODO: maybe add responsive rounded props?
});
</script>
