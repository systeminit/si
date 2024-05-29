<template>
  <div>
    <div class="flex flex-row gap-sm items-center dark:bg-black bg-white">
      <Icon
        name="alert-circle"
        class="text-warning-600 content-center ml-md"
        size="lg"
      />
      <p class="grow py-md">
        The server encountered an error and could not complete your request.
      </p>
    </div>
    <div v-show="show">
      <ol>
        <li>Try again</li>
        <li>Reload the page</li>
        <li>Reach out to us on Discord</li>
      </ol>
    </div>
    <div class="flex flex-row gap-sm items-center">
      <VButton
        label="What can I do?"
        tone="empty"
        variant="solid"
        class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
        @click="() => (show = !show)"
      ></VButton>
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
import { ref } from "vue";
import { Icon } from "@si/vue-lib/design-system";

const emit = defineEmits<{
  (e: "close-toast"): void;
}>();

const show = ref(false);
</script>

<style lang="less">
pre {
  font-family: monospace;
  resize: none;
  display: block;
}
</style>
