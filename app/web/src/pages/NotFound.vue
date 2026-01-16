<template>
  <div class="flex flex-col items-center justify-center h-screen bg-black">
    <meta name="robots" content="noindex" />
    <div class="text-center flex flex-col gap-xs">
      <p class="text-5xl text-white mb-md">Page Not Found</p>
      <p class="text-xl text-white">We tried, though. Seriously.</p>
      <p class="text-xl text-white">Sorry about that, but there is nothing here.</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useHead } from "@vueuse/head";
import { computed } from "vue";

useHead(
  computed(() => ({
    bodyAttrs: {
      class: "overflow-hidden",
    },
  })),
);
</script>
