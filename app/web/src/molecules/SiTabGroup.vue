<template>
  <TabGroup>
    <slot />
    <TabList
      class="border-b-2 dark:border-[#525252] h-11 flex shrink-0 gap-4 px-4"
    >
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
</script>
