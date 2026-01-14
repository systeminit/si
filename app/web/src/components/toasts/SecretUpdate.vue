<template>
  <div
    :class="
      clsx(
        'container flex flex-row gap-xs items-center justify-center p-xs rounded',
        !success ? 'cursor-progress' : 'cursor-pointer',
      )
    "
  >
    <template v-if="!props.success">
      <Icon name="loader" size="xl" tone="action" />
      <p class="text-sm">{{ newSecret ? "Adding" : "Updating" }} secret "{{ props.secretName }}"</p>
    </template>
    <template v-else>
      <Icon name="check" size="xl" tone="success" />
      <p class="text-md">Secret {{ newSecret ? "Added" : "Updated" }}!</p>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";

const props = defineProps<{
  secretName: string;
  success: { type: boolean; default: false };
  newSecret: { type: boolean; default: false };
}>();
</script>
