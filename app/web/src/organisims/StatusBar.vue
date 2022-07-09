<template>
  <Disclosure v-slot="{ open }" as="status" :class="bgColor">
    <!-- NOTE(nick): will likely need to remove max-w-fit -->
    <div class="mx-auto px-4 max-w-fit">
      <div class="flex items-end justify-end h-12">
        <!-- Tabs location -->
        <div class="flex items-center">
          <div class="px-1/2">
            <SiFlyoutTab :kind="SiFlyoutKind.ChangeSet" />
          </div>
          <div class="px-1/2 border-r-2 border-black">
            <SiFlyoutTab :kind="SiFlyoutKind.Qualifications" />
          </div>
        </div>

        <!-- Mobile menu button -->
        <div class="-mr-2 flex sm:hidden">
          <DisclosureButton
            class="inline-flex items-center justify-center p-2 rounded-md text-gray-400 hover:text-white hover:bg-black focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
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
import { MenuIcon, XIcon } from "@heroicons/vue/outline";
import SiFlyoutTab from "@/molecules/SiFlyoutTab.vue";
import { SiFlyoutKind } from "@/molecules/SiFlyoutTab/types";

const bgColor = "bg-[#333333]";
</script>
