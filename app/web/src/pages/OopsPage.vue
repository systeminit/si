<template>
  <div class="flex flex-col items-center justify-center h-screen bg-black overflow-hidden">
    <meta name="robots" content="noindex" />
    <div class="text-center">
      <RichText>
        <p class="text-5xl text-white">Oops!</p>
        <p class="text-xl text-white my-md">Looks like something went wrong with your system.</p>
        <p class="text-xl text-white richtext">
          Please try restarting your SI stack and then
          <a href="/">click here</a>
        </p>
      </RichText>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RichText } from "@si/vue-lib/design-system";
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
