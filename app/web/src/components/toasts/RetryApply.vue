<template>
  <div>
    <div
      class="flex flex-row gap-sm items-center dark:bg-black bg-white max-h-[75vh] overflow-y-auto max-w-lg"
    >
      <Icon
        name="alert-circle"
        class="text-warning-600 content-center ml-md"
        size="lg"
      />
      <p class="grow py-md max-w-lg">
        Error when attemping to apply changes: found dependent values that need
        to be processed. Please retry applying your changes! No further action
        should be required.
      </p>
    </div>
    <div class="flex flex-row gap-sm items-center mt-xs">
      <VButton
        class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
        label="Close"
        tone="empty"
        variant="solid"
        @click="$emit('close-toast')"
      ></VButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { VButton, Icon } from "@si/vue-lib/design-system";

const emit = defineEmits<{
  (e: "close-toast"): void;
}>();
</script>

<style lang="less">
pre {
  font-family: monospace;
  resize: none;
  display: block;
}
</style>
