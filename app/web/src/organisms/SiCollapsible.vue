<template>
  <Disclosure v-slot="{ open }" :as="as">
    <DisclosureButton
      class="flex w-full px-2 py-2 text-left text-sm font-medium focus:outline-none dark:border-neutral-600 border-b-2"
    >
      <ChevronUpIcon
        class="w-5 mr-1.5 dark:text-white flex-shrink-0 block"
        :class="open ? 'rotate-180 transform' : ''"
      />

      <span class="whitespace-nowrap overflow-hidden overflow-ellipsis">{{
        label
      }}</span>
    </DisclosureButton>
    <DisclosurePanel :as="contentAs">
      <slot />
    </DisclosurePanel>
  </Disclosure>
</template>

<script setup lang="ts">
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
import { ChevronUpIcon } from "@heroicons/vue/solid";

defineProps<{
  label: string;
  as?: string;
  contentAs?: string;
}>();
</script>
