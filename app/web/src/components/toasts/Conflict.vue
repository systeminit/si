<template>
  <div>
    <div class="flex flex-row gap-sm items-center dark:bg-black bg-white">
      <Icon
        name="alert-circle"
        class="text-warning-600 content-center ml-md"
        size="lg"
      />
      <p class="grow py-md">
        A conflict occurred. This conflict needs to be addressed before you can
        Apply.
      </p>
    </div>
    <div v-show="show">
      <pre class="text-sm">{{ props.conflict }}</pre>
    </div>
    <div class="flex flex-row gap-sm items-center">
      <VButton
        label="View conflict message"
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
import { Conflict } from "@/store/status.store";

const emit = defineEmits<{
  (e: "close-toast"): void;
}>();

const show = ref(false);

const props = defineProps<{
  conflict: Conflict;
}>();
</script>

<style lang="less">
pre {
  font-family: monospace;
  resize: none;
  display: block;
}
</style>
