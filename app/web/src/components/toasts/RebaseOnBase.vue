<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon name="tools" class="action" />
    <div class="flex flex-col text-sm text-center">Changes from HEAD have been applied to this change set.</div>
    <div class="flex flex-row gap-sm items-center">
      <VButton
        class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
        label="Dismiss"
        tone="empty"
        variant="solid"
        @click="$emit('close-toast')"
      ></VButton>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";

const emit = defineEmits<{
  (e: "close-toast"): void;
}>();
</script>
