<template>
  <div
    :class="
      clsx(
        'container flex flex-row gap-xs items-center justify-center p-xs rounded',
        updating ? 'cursor-progress' : 'cursor-pointer',
      )
    "
  >
    <template v-if="timeout">
      <Icon name="x" size="xl" tone="destructive" />
      <p class="text-sm">Model Status Timeout</p>
    </template>
    <template v-else-if="updating">
      <Icon name="loader" size="xl" tone="action" />
      <p class="text-sm">Updating Model...</p>
    </template>
    <template v-else>
      <Icon name="check-hex-outline" size="xl" tone="success" />
      <p class="text-sm">Model Up-to-Date</p>
    </template>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed } from "vue";
import { useStatusStore } from "@/store/status.store";

const statusStore = useStatusStore();
const updating = computed(() => statusStore.globalStatus.isUpdating);

defineProps({
  timeout: Boolean,
});
</script>
