<script setup lang="ts">
import { inject, computed, type Ref } from 'vue';

const props = defineProps<{
  value: string; // Tab identifier matching the label from DocTabs
}>();

const activeTab = inject<Ref<string>>('activeTab');

// Normalize the value to match tab format (lowercase with hyphens)
const normalizedValue = computed(() =>
  props.value.toLowerCase().replace(/\s+/g, '-')
);

const isActive = computed(() => activeTab?.value === normalizedValue.value);
</script>

<template>
  <div v-show="isActive" class="tab-panel">
    <slot />
  </div>
</template>

<style scoped>
.tab-panel {
  /* Panel is controlled by parent component's content area styling */
}
</style>
