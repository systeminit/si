<template>
  <TabGroup :selected-index="selectedIndex" @change="onChange && onChange">
    <slot />
    <TabList :class="tabListClasses">
      <div
        v-if="startMargin > 0"
        class="border-b border-neutral-300 dark:border-neutral-600"
        :class="'w-' + startMargin"
      ></div>
      <slot name="tabs" />
      <div
        class="grow border-b border-neutral-300 dark:border-neutral-600"
      ></div>
    </TabList>
    <TabPanels as="template">
      <slot name="panels" />
    </TabPanels>
  </TabGroup>
</template>

<script setup lang="ts">
import { TabGroup, TabPanels, TabList } from "@headlessui/vue";

withDefaults(
  defineProps<{
    selectedIndex?: number;
    onChange?: (_index: number) => void;
    tabListClasses?: string;
    startMargin?: number;
  }>(),
  {
    selectedIndex: undefined,
    onChange: undefined,
    tabListClasses:
      "h-11 flex shrink-0 w-full bg-white dark:bg-neutral-800 sticky top-0 z-50 overflow-hidden",
    startMargin: 0,
  },
);
</script>
