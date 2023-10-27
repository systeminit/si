<template>
  <div
    class="p-xs flex-none flex flex-col border-b border-neutral-200 dark:border-neutral-600"
  >
    <div class="text-lg font-bold text-center">
      {{ title }}
    </div>
    <div
      v-if="subtitle"
      ref="addingSecretNameRef"
      v-tooltip="subtitleTooltip"
      class="text-center text-sm italic text-neutral-400 break-words line-clamp-3"
    >
      {{ subtitle }}
    </div>
  </div>
</template>

<script lang="ts" setup>
const props = defineProps({
  title: { type: String, required: true },
  subtitle: { type: String },
  subtitleTooltip: { type: Object },
});
</script>
