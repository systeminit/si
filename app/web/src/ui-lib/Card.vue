<template>
  <div
    :class="
      clsx('bg-neutral-900 p-l shadow text-white', rounded && 'rounded-sm')
    "
  >
    <slot />

    <hr v-if="$slots.footer" class="border border-shade-100 -mx-l my-l" />
    <slot name="footer" />
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";

const props = defineProps({
  rounded: Boolean,
  // TODO: add padding size props (responsive)
  // TODO: maybe add responsive rounded props?
});
</script>
