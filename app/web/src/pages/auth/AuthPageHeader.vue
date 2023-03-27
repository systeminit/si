<template>
  <div class="dark:text-neutral-200 flex flex-row gap-md px-lg">
    <img
      class="h-16 shadow-[0_1px_8px_1px_rgba(0,0,0,0.3)] rounded-lg"
      :src="SiLogoUrl"
      alt="System Initiative"
    />
    <Stack spacing="sm">
      <h2 class="text-3xl font-extrabold capsize mt-2xs">{{ title }}</h2>
      <RichText class="capsize">
        <slot />
      </RichText>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import { Stack, RichText } from "@si/vue-lib/design-system";
import SiLogoUrl from "@si/vue-lib/brand-assets/si-logo.svg?url";

const props = defineProps({
  title: { type: String, required: true },
});
</script>
