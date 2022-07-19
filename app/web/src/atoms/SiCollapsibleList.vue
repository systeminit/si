<template>
  <Disclosure v-slot="{ open }" :as="as" class="mt-2">
    <DisclosureButton
      class="flex w-full justify-between rounded-lg bg-purple-100 px-4 py-2 text-left text-sm font-medium text-purple-900 hover:bg-purple-200 focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75"
    >
      <span :class="open ? 'rotate-180 transform' : ''" class="text-purple-500">
        v
      </span>
      <span>{{ label }}</span>
    </DisclosureButton>
    <DisclosurePanel as="ul" class="px-4 pt-4 pb-2 text-sm text-gray-500">
      <slot />
    </DisclosurePanel>
  </Disclosure>
</template>

<script setup lang="ts">
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";

defineProps<{
  label: string;
  as?: string;
}>();
</script>
