<template>
  <div class="flex flex-col items-center gap-sm text-center">
    <div
      :class="
        clsx(
          !iconNoBg && [
            'mt-sm p-sm rounded-full',
            themeClasses('bg-neutral-200', 'bg-neutral-700'),
          ],
        )
      "
    >
      <Icon :name="icon" :size="iconSize" />
    </div>
    <div :class="themeClasses('text-neutral-800', 'text-neutral-200')">
      {{ text }}
    </div>
    <slot name="secondary">
      <div
        v-if="secondaryText"
        :class="themeClasses('text-neutral-600', 'text-neutral-400')"
      >
        {{ secondaryText }}
      </div>
    </slot>
  </div>
</template>

<script setup lang="ts">
import {
  Icon,
  IconNames,
  IconSizes,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType } from "vue";

defineProps({
  text: { type: String, required: true },
  secondaryText: { type: String },
  icon: { type: String as PropType<IconNames>, required: true },
  iconSize: { type: String as PropType<IconSizes> },
  iconNoBg: { type: Boolean },
});
</script>
