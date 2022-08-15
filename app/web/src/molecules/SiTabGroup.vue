<template>
  <TabGroup :selected-index="selectedIndex" @change="props.onChange">
    <slot />
    <TabList :class="tabListClasses">
      <!-- TODO(victor): This tabbed content should scroll horizontally on overflow and never wrap, probably-->
      <slot name="tabs" />
    </TabList>
    <TabPanels as="template">
      <slot name="panels" />
    </TabPanels>
  </TabGroup>
</template>

<script setup lang="ts">
import { TabGroup, TabPanels, TabList } from "@headlessui/vue";

const props = withDefaults(
  defineProps<{
    selectedIndex?: number;
    onChange?: (_index: number) => void;
    tabListClasses?: string;
  }>(),
  {
    selectedIndex: undefined,
    onChange: undefined,
    tabListClasses:
      "border-b-2 dark:border-neutral-600 h-11 flex shrink-0 gap-4 px-4 w-full",
  },
);
</script>
