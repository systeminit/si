<template>
  <div class="p-2">
    <input placeholder="search" />

    <p>
      Get Started by dragging the assets that you wish to include in your
      application into the canvas to the right
    </p>

    <ul>
      <Disclosure v-slot="{ open }" as="li" class="mt-2">
        <DisclosureButton
          class="flex w-full justify-between rounded-lg bg-purple-100 px-4 py-2 text-left text-sm font-medium text-purple-900 hover:bg-purple-200 focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75"
        >
          <span
            :class="open ? 'rotate-180 transform' : ''"
            class="text-purple-500"
          >
            v
          </span>
          <span>Type 1</span>
        </DisclosureButton>
        <DisclosurePanel as="ul" class="px-4 pt-4 pb-2 text-sm text-gray-500">
          <SiListItem />
          <SiListItem />
          <SiListItem />
        </DisclosurePanel>
      </Disclosure>
      <Disclosure v-slot="{ open }" as="li" class="mt-2">
        <DisclosureButton
          class="flex w-full justify-between rounded-lg bg-purple-100 px-4 py-2 text-left text-sm font-medium text-purple-900 hover:bg-purple-200 focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75"
        >
          <span
            :class="open ? 'rotate-180 transform' : ''"
            class="text-purple-500"
          >
            v
          </span>
          <span>Type 2</span>
        </DisclosureButton>
        <DisclosurePanel as="ul" class="px-4 pt-4 pb-2 text-sm text-gray-500">
          <SiListItem />
          <SiListItem />
          <SiListItem />
        </DisclosurePanel>
      </Disclosure>
    </ul>
  </div>
</template>

<script setup lang="ts">
import SiListItem from "@/molecules/SiListItem.vue";
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
</script>
