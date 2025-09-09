<template>
  <div
    :class="
      clsx(
        'rounded border',
        themeClasses(
          'bg-white border-neutral-300',
          'bg-neutral-800 border-neutral-600',
        ),
      )
    "
  >
    <div
      :class="
        clsx(
          'p-sm select-none rounded-t border-b',
          themeClasses('border-neutral-300', 'border-neutral-600'),
        )
      "
    >
      <slot name="header" />
    </div>
    <div class="flex flex-col p-sm gap-md">
      <slot name="body" />
    </div>
    <div class="flex flex-row justify-between gap-xs px-sm py-xs">
      <div class="flex flex-row gap-xs">
        <slot name="footerLeft" />
      </div>
      <div class="flex flex-row gap-xs">
        <slot name="footerRight" />
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
</script>
