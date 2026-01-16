<template>
  <NewButton
    icon="dots-vertical"
    :disabled="disabled"
    tone="empty"
    :class="
      selected
        ? themeClasses('bg-action-500', 'bg-action-300')
        : themeClasses('hover:bg-neutral-200 active:bg-action-500', 'hover:bg-neutral-600 active:bg-action-300')
    "
  />
</template>

<script lang="ts" setup>
import { NewButton, themeClasses } from "@si/vue-lib/design-system";

defineProps({
  selected: { type: Boolean },
  disabled: { type: Boolean, default: false },
});
</script>
