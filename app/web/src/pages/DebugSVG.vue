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

      <div class="w-full px-lg text-xl text-left">Other SVGs</div>
      <div
        class="w-full flex flex-row flex-wrap px-lg pb-lg justify-start gap-sm items-center"
      >
        <NodeSkeleton
          v-tooltip="'NodeSkeleton'"
          color="#ff00ff"
          class="cursor-pointer"
        />
        <SiLogo v-tooltip="'SiLogo'" class="cursor-pointer h-xl w-xl" />
        <CheechSvg
          v-tooltip="'CheechSvg'"
          class="h-xl w-xl rounded-full bg-shade-0 border-2 border-shade-100 cursor-pointer"
        />
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { Icon, IconNames, ICONS, LOGO_ICONS } from "@si/vue-lib/design-system";
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";
import CheechSvg from "@/assets/images/cheech-and-chong.svg?component";
import EmptyStateIcon, { BIG_ICONS } from "@/components/EmptyStateIcon.vue";
import AppLayout from "@/components/layout/AppLayout.vue";
import DebugNavbar from "@/components/DebugNavbar.vue";
import NodeSkeleton from "@/components/NodeSkeleton.vue";
</script>
