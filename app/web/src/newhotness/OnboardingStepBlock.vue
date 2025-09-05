<template>
  <div class="bg-neutral-800 rounded border border-neutral-600">
    <div
      :class="
        clsx(
          'p-sm border-neutral-600 select-none rounded-t',
          'border-b',
        )
      "
    >
      <slot name="header" />
    </div>
    <div class="flex flex-col p-sm gap-md">
      <slot name="body" />
    </div>
    <div class="flex flex-row justify-end gap-xs px-sm py-xs">
      <slot name="footer" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
</script>
