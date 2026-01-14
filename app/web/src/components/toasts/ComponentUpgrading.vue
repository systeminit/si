<template>
  <div class="container flex flex-row gap-xs items-center justify-center rounded cursor-progress">
    <template v-if="!props.success">
      <Icon name="loader" size="md" tone="action" />
      <p class="text-md">Updating "{{ props.componentName }}"</p>
    </template>
    <template v-else>
      <Icon name="check-hex-outline" size="md" tone="success" />
      <p class="text-md">Component up-to-date</p>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";

const props = defineProps<{
  componentName: string;
  success: { type: boolean; default: false };
}>();
</script>
