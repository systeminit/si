<template>
  <Disclosure v-slot="{ open }" :as="as" :default-open="defaultOpen">
    <DisclosureButton
      class="flex w-full px-2 py-2 text-left text-sm font-medium focus:outline-none dark:border-neutral-600 border-b"
    >
      <Icon
        :name="open ? 'chevron--up' : 'chevron--down'"
        size="s"
        class="mr-1.5 dark:text-white flex-shrink-0 block"
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
import { computed, useSlots } from "vue";
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
import Icon from "@/ui-lib/Icon.vue";

defineProps<{
  label?: string;
  as?: string;
  contentAs?: string;
  defaultOpen?: boolean;
}>();

const slots = useSlots();
const labelSlot = computed(() => slots.label?.());
</script>
