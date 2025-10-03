<script setup lang="ts">
import { ref, provide, onMounted } from 'vue';

interface Tab {
  label: string;
  value: string;
}

const props = defineProps<{
  tabs: string; // Comma-separated list like "Web Application,AI Agent,Public API"
  defaultTab?: string;
}>();

// Parse tabs from comma-separated string
const tabList = ref<Tab[]>(
  props.tabs.split(',').map((tab) => {
    const trimmed = tab.trim();
    return {
      label: trimmed,
      value: trimmed.toLowerCase().replace(/\s+/g, '-'),
    };
  })
);

const activeTab = ref(
  props.defaultTab?.toLowerCase().replace(/\s+/g, '-') || tabList.value[0]?.value || ''
);

// Provide active tab to child TabPanel components
provide('activeTab', activeTab);
provide('setActiveTab', (value: string) => {
  activeTab.value = value;
});
</script>

<template>
  <div class="doc-tabs">
    <div class="doc-tabs-nav">
      <button
        v-for="tab in tabList"
        :key="tab.value"
        :class="['doc-tab-button', { active: activeTab === tab.value }]"
        @click="activeTab = tab.value"
      >
        {{ tab.label }}
      </button>
    </div>
    <div class="doc-tabs-content">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.doc-tabs {
  margin: 1.5rem 0;
  border: 1px solid var(--vp-c-divider);
  border-radius: 8px;
  overflow: hidden;
}

.doc-tabs-nav {
  display: flex;
  gap: 0;
  background-color: var(--vp-c-bg-soft);
  border-bottom: 1px solid var(--vp-c-divider);
  padding: 0;
  overflow-x: auto;
}

.doc-tab-button {
  padding: 0.75rem 1.5rem;
  border: none;
  background: transparent;
  color: var(--vp-c-text-2);
  font-size: 0.9rem;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.2s ease;
  border-bottom: 2px solid transparent;
  position: relative;
}

.doc-tab-button:hover {
  color: var(--vp-c-brand-1);
  background-color: var(--vp-c-default-soft);
}

.doc-tab-button.active {
  color: var(--vp-c-brand-1);
  border-bottom-color: var(--vp-c-brand-1);
}

.doc-tabs-content {
  padding: 1.5rem;
  background-color: var(--vp-c-bg);
}

/* Ensure markdown content inside tabs renders properly */
.doc-tabs-content :deep(h1),
.doc-tabs-content :deep(h2),
.doc-tabs-content :deep(h3),
.doc-tabs-content :deep(h4),
.doc-tabs-content :deep(h5),
.doc-tabs-content :deep(h6) {
  margin-top: 1.5rem;
  margin-bottom: 0.5rem;
}

.doc-tabs-content :deep(h1:first-child),
.doc-tabs-content :deep(h2:first-child),
.doc-tabs-content :deep(h3:first-child),
.doc-tabs-content :deep(h4:first-child),
.doc-tabs-content :deep(h5:first-child),
.doc-tabs-content :deep(h6:first-child) {
  margin-top: 0;
}

.doc-tabs-content :deep(p) {
  margin: 1rem 0;
  line-height: 1.7;
}

.doc-tabs-content :deep(ul),
.doc-tabs-content :deep(ol) {
  padding-left: 1.5rem;
  margin: 1rem 0;
}

.doc-tabs-content :deep(pre) {
  margin: 1rem 0;
}

.doc-tabs-content :deep(code) {
  font-size: 0.9em;
}

.doc-tabs-content :deep(blockquote) {
  margin: 1rem 0;
  padding-left: 1rem;
  border-left: 4px solid var(--vp-c-divider);
  color: var(--vp-c-text-2);
}

.doc-tabs-content :deep(table) {
  margin: 1rem 0;
}

.doc-tabs-content :deep(img) {
  max-width: 100%;
  height: auto;
  margin: 1rem 0;
}
</style>
