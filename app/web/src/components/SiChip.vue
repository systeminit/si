<template>
  <div
    v-if="text"
    :class="
      clsx(
        'inline-block px-2xs whitespace-nowrap font-bold',
        variant === 'classic' && 'rounded border-solid border',
        variant === 'simple' && 'rounded-sm',
        uppercase && 'uppercase',
        textSize && `text-${textSize}`,
        colorClasses,
      )
    "
  >
    <slot>{{ text }}</slot>
  </div>
</template>

<script lang="ts" setup>
import { FontSizes, Tones, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, computed } from "vue";

export type ChipVariant = "classic" | "simple";

const props = defineProps({
  text: { type: String },
  textSize: { type: String as PropType<FontSizes> },
  tone: { type: String as PropType<Tones>, default: "action" },
  uppercase: { type: Boolean },
  variant: { type: String as PropType<ChipVariant>, default: "classic" },
});

const colorClasses = computed(() => {
  if (props.variant === "simple") {
    switch (props.tone) {
      case "warning":
        return themeClasses("bg-warning-500 text-shade-0", "bg-warning-600 text-shade-100");
      // TODO - implement other tones here as needed!
      default: // action is default
        return themeClasses("bg-action-500 text-shade-0", "bg-action-200 text-shade-100");
    }
  }
  // 'classic'
  switch (props.tone) {
    case "warning":
      return "border-warning-700 text-warning-700 bg-warning-100";
    // TODO - implement other tones here as needed!
    default: // action is default
      return "border-action-700 text-action-700 bg-action-100";
  }
});
</script>
