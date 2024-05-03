<template>
  <AppLayout pageMode="scroll">
    <meta name="robots" content="noindex" />
    <DebugNavbar />
    <div
      class="flex flex-col pt-sm gap-sm items-center justify-center border-t border-white"
    >
      <div class="w-full px-lg text-xl text-left">Main Icon Set</div>
      <div
        class="w-full flex flex-row flex-wrap gap-sm px-lg pb-lg justify-start"
      >
        <Icon
          v-for="(_, name) in ICONS"
          :key="name"
          v-tooltip="name"
          :name="name as IconNames"
          class="cursor-pointer"
        />
      </div>

      <div class="w-full px-lg text-xl text-left">Logo Icons</div>
      <div
        class="w-full flex flex-row flex-wrap gap-sm px-lg pb-lg justify-start"
      >
        <Icon
          v-for="(_, name) in LOGO_ICONS"
          :key="name"
          v-tooltip="name"
          :name="name as IconNames"
          class="cursor-pointer"
        />
      </div>

      <div class="w-full px-lg text-xl text-left">Empty State Icons</div>
      <div class="w-full flex flex-row flex-wrap px-lg pb-lg justify-start">
        <div v-for="(_, name) in BIG_ICONS" :key="name" class="basis-1/4">
          <EmptyStateIcon
            v-tooltip="name"
            :name="name"
            class="w-full cursor-pointer"
          />
        </div>
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { Icon, IconNames, ICONS, LOGO_ICONS } from "@si/vue-lib/design-system";
import EmptyStateIcon, { BIG_ICONS } from "@/components/EmptyStateIcon.vue";
import AppLayout from "@/components/layout/AppLayout.vue";
import DebugNavbar from "@/components/DebugNavbar.vue";
</script>
