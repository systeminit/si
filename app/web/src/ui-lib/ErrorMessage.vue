<template>
  <div
    v-if="props.message"
    class="border border-destructive-500 text-destructive-400 p-xs text-sm rounded-sm flex flex-row items-center"
  >
    <Icon name="alert-triangle" class="mr-xs flex-none" />
    <div class="flex-grow">{{ message }}</div>
  </div>
</template>

<script lang="ts" setup>
import Icon from "./Icon.vue";

const props = defineProps({
  message: { type: String },
});
</script>
