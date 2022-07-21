<template>
  <Tab v-slot="{ selected }" class="focus:outline-none">
    <span
      class="text-sm"
      :class="selected ? 'font-bold text-black h-full' : 'text-gray-400'"
    >
      <slot />
    </span>
  </Tab>
</template>

<script setup lang="ts">
import { Tab } from "@headlessui/vue";
</script>
