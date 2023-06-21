<template>
  <div class="dark:text-neutral-200 flex flex-row gap-md px-lg">
    <SiLogo class="h-16" />
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
import SiLogo from "@si/vue-lib/brand-assets/si-logo-symbol.svg?component";

const props = defineProps({
  title: { type: String, required: true },
});
</script>
