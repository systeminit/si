<template>
  <div
    :class="
      clsx(
        'flex flex-row items-center gap-xs border w-fit px-3xs rounded-sm',
        'relative',
        'max-w-full',
        themeClasses(
          'text-neutral-600 border-neutral-400 bg-neutral-100',
          'text-neutral-400 border-neutral-600 bg-neutral-900',
        ),
      )
    "
    @mouseenter="show = true"
    @mouseleave="show = false"
  >
    <slot />
    <div
      v-if="show"
      :class="
        clsx(
          'absolute z-[500] bottom-[-75px] left-[-17px] w-[300px]',
          themeClasses(
            'text-neutral-600 border-neutral-400 bg-neutral-100',
            'text-neutral-400 border-neutral-600 bg-neutral-800',
          ),
        )
      "
    >
      <slot name="components" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";

const show = ref(false);

defineProps<{
  tooltip?: { content: string; theme: string };
}>();
</script>
