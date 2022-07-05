<template>
  <Disclosure v-slot="{ open }" as="nav" class="bg-gray-800">
    <div class="mx-auto px-4 sm:px-4 lg:px-4">
      <div class="flex items-center justify-between h-16">
        <!-- Left side -->
        <div class="flex-shrink-0">
          <img
            class="block lg:hidden h-8 w-auto"
            :src="SiLogoWts"
            alt="SI Logo"
          />
          <img
            class="hidden lg:block h-8 w-auto"
            :src="SiLogoWts"
            alt="SI Logo"
          />
        </div>

        <!-- Center -->
        <div class="flex items-center">
          <div class="hidden sm:block sm:ml-6">
            <div class="flex space-x-4">
              <button
                type="button"
                class="px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
              >
                <span class="sr-only">View notifications</span>
                <CollectionIcon class="h-6 w-6" aria-hidden="true" />
              </button>

              <button
                type="button"
                class="px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
              >
                <span class="sr-only">View notifications</span>
                <BeakerIcon class="h-6 w-6" aria-hidden="true" />
              </button>

              <!-- Vertical bar -->
              <div class="h-8 w-1 bg-gray-400"></div>

              <button
                type="button"
                class="px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
              >
                <span class="sr-only">View notifications</span>
                <EyeIcon class="h-6 w-6" aria-hidden="true" />
              </button>

              <button
                type="button"
                class="px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
              >
                <span class="sr-only">View notifications</span>
                <PlayIcon class="h-6 w-6" aria-hidden="true" />
              </button>
            </div>
          </div>
        </div>

        <!-- Right side -->
        <div class="hidden sm:ml-6 sm:block">
          <div class="flex items-center">
            <button
              type="button"
              class="text-sm px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
            >
              <span class="sr-only">View notifications</span>
              100%
            </button>

            <button
              type="button"
              class="px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
            >
              <span class="sr-only">View notifications</span>
              <LinkIcon class="h-6 w-6" aria-hidden="true" />
            </button>

            <button
              type="button"
              class="px-2 py-1 rounded-full text-gray-300 hover:text-white focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-800 focus:ring-white"
            >
              <span class="sr-only">View notifications</span>
              <MoonIcon class="h-6 w-6" aria-hidden="true" />
            </button>

            <SiProfile :enable-old-app-switch="true" />
          </div>
        </div>

        <!-- Mobile menu button -->
        <div class="-mr-2 flex sm:hidden">
          <DisclosureButton
            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
          >
            <span class="sr-only">Open main menu</span>
            <MenuIcon v-if="!open" class="block h-6 w-6" aria-hidden="true" />
            <XIcon v-else class="block h-6 w-6" aria-hidden="true" />
          </DisclosureButton>
        </div>
      </div>
    </div>
  </Disclosure>
</template>

<script setup lang="ts">
import { Disclosure, DisclosureButton } from "@headlessui/vue";
import {
  EyeIcon,
  MenuIcon,
  XIcon,
  MoonIcon,
  LinkIcon,
} from "@heroicons/vue/outline";
import { PlayIcon, BeakerIcon, CollectionIcon } from "@heroicons/vue/solid";
import SiProfile from "@/molecules/SiProfile.vue";
import SiLogoWts from "@/assets/images/si-logo-wts.svg";
</script>
