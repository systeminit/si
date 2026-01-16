<template>
  <div class="flex-1 gap-sm overflow-hidden flex flex-col items-center justify-center">
    <div class="grow flex items-center justify-center">
      <div class="flex flex-col items-center justify-center gap-xs text-center max-w-lg">
        <div :class="clsx('p-sm rounded-full', themeClasses('bg-neutral-300', 'bg-neutral-700'))">
          <Icon name="arrow-outward" />
        </div>
        <p :class="clsx('text-sm font-medium', themeClasses('text-neutral-800', 'text-neutral-200'))">
          No default subscriptions yet
        </p>
        <p :class="clsx('text-xs leading-relaxed', themeClasses('text-neutral-600', 'text-neutral-400'))">
          Default subscriptions are values that, once set, automatically apply to all new components with the same
          schema.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
</script>
