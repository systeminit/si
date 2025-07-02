<template>
  <div
    :class="
      clsx(
        'component tile',
        'cursor-pointer border rounded overflow-hidden h-[233px] flex flex-col',
        themeClasses(
          'bg-shade-0 border-neutral-400',
          'bg-neutral-900 border-neutral-600',
        ),
      )
    "
  >
    <div
      :class="
        clsx(
          'skeleton-shimmer flex-none h-16 border-b',
          themeClasses(
            'bg-neutral-100 border-neutral-300',
            'bg-neutral-800 border-neutral-600',
          ),
        )
      "
    ></div>

    <div
      :class="
        clsx(
          'skeleton-shimmer flex-1 border-b',
          themeClasses(
            'bg-neutral-50 border-neutral-300',
            'bg-neutral-850 border-neutral-600',
          ),
        )
      "
    ></div>

    <div
      :class="
        clsx(
          'skeleton-shimmer flex-none h-12',
          themeClasses('bg-neutral-100', 'bg-neutral-800'),
        )
      "
    ></div>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
</script>
