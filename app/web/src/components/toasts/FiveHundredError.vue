<template>
  <div>
    <div class="flex flex-row gap-sm items-center dark:bg-black bg-white max-h-[75vh] overflow-y-auto max-w-lg">
      <Icon name="alert-circle" class="text-warning-600 content-center ml-md" size="lg" />
      <p class="grow py-md max-w-lg">
        The server encountered an error and could not complete your request. Please reach out on Discord if you need
        some help! <br />
        <br />
        Request to
        <span class="font-bold">{{ requestUrl }}</span> returned:
      </p>
    </div>
    <ErrorMessage class="mx-1" icon="alert-triangle" tone="warning">
      <div class="break-words max-w-lg">{{ message }}</div>
    </ErrorMessage>
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
import { VButton, Icon, ErrorMessage } from "@si/vue-lib/design-system";

const emit = defineEmits<{
  (e: "close-toast"): void;
}>();

const props = defineProps({
  requestUrl: { type: String, required: false },
  message: { type: String, required: false },
});
</script>

<style lang="less">
pre {
  font-family: monospace;
  resize: none;
  display: block;
}
</style>
