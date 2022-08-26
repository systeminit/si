<template>
  <Disclosure v-slot="{ open }" :as="as" :default-open="defaultOpen">
    <DisclosureButton
      class="flex w-full px-2 py-2 text-left text-sm font-medium focus:outline-none dark:border-neutral-600 border-b"
    >
      <ChevronUpIcon
        class="w-5 mr-1.5 dark:text-white flex-shrink-0 block"
        :class="open ? 'rotate-180 transform' : ''"
      />

      <slot name="label" />
      <span
        v-if="labelSlot === undefined"
        class="whitespace-nowrap overflow-hidden overflow-ellipsis"
      >
        {{ label }}
      </span>
    </DisclosureButton>
    <DisclosurePanel :as="contentAs">
      <slot />
    </DisclosurePanel>
  </Disclosure>
</template>

<script setup lang="ts">
import { Disclosure, DisclosureButton, DisclosurePanel } from "@headlessui/vue";
import { ChevronUpIcon } from "@heroicons/vue/solid";
import { computed, useSlots } from "vue";

defineProps<{
  label?: string;
  as?: string;
  contentAs?: string;
  defaultOpen?: boolean;
}>();

const slots = useSlots();
const labelSlot = computed(() => slots.label?.());
</script>
