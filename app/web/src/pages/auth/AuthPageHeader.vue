<template>
  <div class="dark:text-neutral-200 flex flex-row gap-md px-lg">
    <img
      class="h-16 shadow-[0_1px_8px_1px_rgba(0,0,0,0.3)] rounded-lg"
      :src="siLogoWts"
      alt="System Initiative"
    />
    <Stack spacing="md">
      <h2 class="text-4xl font-extrabold capsize">{{ title }}</h2>
      <RichText class="capsize">
        {{ altActionText }}
        <router-link :to="{ name: altActionRouteName }">{{
          altActionLinkText
        }}</router-link>
      </RichText>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import Stack from "@/ui-lib/layout/Stack.vue";
import RichText from "@/ui-lib/RichText.vue";
import siLogoWts from "@/assets/images/si-logo-wts.svg?url";

const props = defineProps({
  title: { type: String, required: true },
  altActionText: { type: String, required: true },
  altActionLinkText: { type: String, required: true },
  altActionRouteName: { type: String, required: true },
});
</script>
