<template>
  <PopoverPanel
    class="absolute z-10 left-1/2 transform -translate-x-1/2 mt-3 px-2 w-screen max-w-xs sm:px-0"
  >
    <div
      class="rounded-lg shadow-lg ring-1 ring-black ring-opacity-5 overflow-hidden"
    >
      <div class="relative grid gap-6 bg-white px-5 py-6 sm:gap-8 sm:p-8">
        <a
          v-for="item in items"
          :key="item.name"
          :href="item.href"
          class="-m-3 p-3 block rounded-md hover:bg-gray-50 transition ease-in-out duration-150"
        >
          <p class="text-base font-medium text-gray-900">
            {{ item.name }}
          </p>
          <p class="mt-1 text-sm text-gray-500">
            {{ item.description }}
          </p>
        </a>
      </div>
    </div>
  </PopoverPanel>
</template>

<script setup lang="ts">
import { PopoverPanel } from "@headlessui/vue";

const items = [
  {
    name: "Foo",
    description: "Foo foo foo foo foo.",
    href: "#",
  },
  {
    name: "Bar",
    description: "Bar bar bar bar bar.",
    href: "#",
  },
];
</script>
