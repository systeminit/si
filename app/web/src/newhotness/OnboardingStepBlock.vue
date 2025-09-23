<template>
  <div
    :class="
      clsx(
        'rounded border w-full flex flex-col grow',
        themeClasses(
          'bg-white border-neutral-400',
          'bg-neutral-800 border-neutral-600',
        ),
      )
    "
  >
    <div
      :class="
        clsx(
          'p-sm select-none rounded-t border-b text-sm',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
    >
      <slot name="header" />
    </div>
    <div class="flex flex-col p-sm gap-md grow text-sm">
      <slot name="body" />
    </div>
    <div
      :class="
        clsx(
          'flex flex-row justify-between gap-xs px-sm py-xs border-t',
          themeClasses('border-neutral-400', 'border-neutral-600'),
        )
      "
    >
      <div class="flex flex-row gap-xs items-center">
        <slot name="footerLeft" />
      </div>
      <div class="flex flex-row gap-xs items-center">
        <slot name="footerRight" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
</script>
