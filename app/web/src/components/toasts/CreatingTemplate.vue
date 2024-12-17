<template>
  <div
    :class="
      clsx(
        'container flex flex-row gap-xs items-center justify-center p-xs rounded',
        updating ? 'cursor-progress' : 'cursor-pointer',
      )
    "
  >
    <template v-if="updating">
      <Icon name="loader" size="xl" tone="action" />
      <p class="text-sm">Creating Template...</p>
    </template>
    <template v-else>
      <Icon name="check-hex-outline" size="xl" tone="success" />
      <p class="text-sm">
        Template {{ templateName ? `${templateName} ` : "" }}Created
      </p>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps({
  updating: { type: Boolean },
  templateName: { type: String },
});
</script>
