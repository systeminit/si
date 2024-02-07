<template>
  <div
    class="flex text-neutral-500 dark:text-neutral-400 border-b dark:border-neutral-500 items-center px-xs py-sm gap-xs"
  >
    <div class="flex-none empty:hidden">
      <slot name="icon">
        <Icon v-if="icon" :name="icon" />
      </slot>
    </div>

    <div class="grow uppercase text-md font-bold leading-6">
      <slot name="label">{{ label }}</slot>
    </div>
    <div class="flex-none empty:hidden">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, IconNames } from "@si/vue-lib/design-system";
import { PropType } from "vue";

const props = defineProps({
  label: { type: String },
  icon: { type: String as PropType<IconNames> },
});
</script>
