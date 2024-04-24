<template>
  <div class="flex flex-col gap-md pt-md items-center justify-center bg-black">
    <meta name="robots" content="noindex" />

    <div class="w-full px-lg text-xl text-left">Main Icon Set</div>
    <div class="w-full flex flex-row flex-wrap gap-sm px-lg justify-start">
      <Icon v-for="(_, name) in ICONS" :key="name" :name="name as IconNames" />
    </div>

    <div class="w-full px-lg text-xl text-left">Logo Icons</div>
    <div class="w-full flex flex-row flex-wrap gap-sm px-lg justify-start">
      <Icon
        v-for="(_, name) in LOGO_ICONS"
        :key="name"
        :name="name as IconNames"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Icon, IconNames, ICONS, LOGO_ICONS } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { computed } from "vue";

useHead(
  computed(() => ({
    bodyAttrs: {
      class: "bg-black overflow-hidden",
    },
  })),
);
</script>
