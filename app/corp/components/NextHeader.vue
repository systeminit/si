<template>
  <div class="relative pt-6 pb-16 sm:pb-24">
    <Popover>
      <div class="max-w-7xl mx-auto px-4 sm:px-6">
        <nav
          class="relative flex items-center justify-between sm:h-10 md:justify-center"
          aria-label="Global"
        >
          <div
            class="flex items-center flex-1 md:absolute md:inset-y-0 md:left-0"
          >
            <div class="flex items-center justify-between w-full md:w-auto">
              <a href="/">
                <span class="sr-only">System Initiative</span>
                <img
                  class="h-8 w-auto sm:h-10"
                  src="~/assets/si-logo-wts.svg"
                  alt="SI"
                />
              </a>
              <div class="-mr-2 flex items-center md:hidden">
                <PopoverButton
                  class="bg-neutral-700 rounded-md p-2 inline-flex items-center justify-center text-gray-400 hover:text-gray-400 hover:bg-neutral-600 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-neutral-500"
                >
                  <span class="sr-only">Open main menu</span>
                  <MenuIcon class="h-6 w-6" aria-hidden="true" />
                </PopoverButton>
              </div>
            </div>
          </div>
          <div class="hidden md:flex md:space-x-10">
            <a
              v-for="item in navigation"
              :key="item.name"
              :href="item.href"
              class="font-medium text-gray-300 hover:text-emerald-100"
              >{{ item.name }}</a
            >
          </div>
          <div
            class="hidden md:absolute md:flex md:items-center md:justify-end md:inset-y-0 md:right-0"
          >
            <span class="inline-flex rounded-md shadow">
              <a
                href="https://app.systeminit.com"
                class="inline-flex items-center px-4 py-2 border border-transparent text-base font-medium rounded-sm text-neutral-100 bg-neutral-900 hover:text-emerald-400"
              >
                Log in
              </a>
            </span>
          </div>
        </nav>
      </div>

      <transition
        enter-active-class="duration-150 ease-out"
        enter-from-class="opacity-0 scale-95"
        enter-to-class="opacity-100 scale-100"
        leave-active-class="duration-100 ease-in"
        leave-from-class="opacity-100 scale-100"
        leave-to-class="opacity-0 scale-95"
      >
        <PopoverPanel
          focus
          class="absolute z-10 top-0 inset-x-0 p-2 transition transform origin-top-right md:hidden"
        >
          <div
            class="rounded-lg shadow-md bg-neutral-900 ring-1 ring-black ring-opacity-5 overflow-hidden"
          >
            <div class="px-5 pt-4 flex items-center justify-between">
              <div>
                <img class="h-8 w-auto" src="~/assets/si-logo-wts.svg" alt="" />
              </div>
              <div class="-mr-2">
                <PopoverButton
                  class="bg-neutral-900 rounded-md p-2 inline-flex items-center justify-center text-gray-400 hover:text-gray-500 hover:bg-neutral-800 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-neutral-500"
                >
                  <span class="sr-only">Close main menu</span>
                  <XIcon class="h-6 w-6" aria-hidden="true" />
                </PopoverButton>
              </div>
            </div>
            <div class="px-2 pt-2 pb-3 space-y-1">
              <a
                v-for="item in navigation"
                :key="item.name"
                :href="item.href"
                class="block px-3 py-2 rounded-md text-base font-medium text-gray-300 hover:text-emerald-100"
                >{{ item.name }}</a
              >
            </div>
            <a
              href="https://app.systeminit.com"
              class="block w-full px-5 py-3 text-center font-medium text-neutral-300 bg-neutral-700 hover:bg-neutral-600 hover:text-emerald-300"
            >
              Log in
            </a>
          </div>
        </PopoverPanel>
      </transition>
    </Popover>

    <div
      v-if="props.heroWords"
      id="product"
      class="mt-12 mx-auto max-w-7xl px-4 sm:mt-24 sm:px-6"
    >
      <div class="text-center">
        <h1
          class="text-4xl tracking-tight font-extrabold text-sky-200 sm:text-5xl md:text-6xl"
        >
          <span class="block">DevOps without papercuts.</span>
        </h1>
        <p
          class="mt-3 max-w-md mx-auto text-base text-gray-200 sm:text-lg md:mt-5 md:text-xl md:max-w-3xl"
        >
          A real time, collaborative power tool for DevOps Teams.
        </p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Popover, PopoverButton, PopoverPanel } from "@headlessui/vue";
import { MenuIcon, XIcon } from "@heroicons/vue/outline";
const props = defineProps<{ heroWords: boolean }>();

const navigation = [
  { name: "Product", href: "/#product" },
  { name: "Features", href: "/#features" },
  { name: "Blog", href: "/blog" },
  { name: "Jobs", href: "https://systeminit.pinpointhq.com/" },
];
</script>
