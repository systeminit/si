<template>
  <div
    v-if="status !== 'operational' && status !== 'unknown'"
    :class="
      clsx(
        'flex items-center gap-xs px-xs py-2xs rounded border text-xs',
        themeClasses(
          'bg-warning-100 text-neutral-800 border-warning-400',
          'bg-[rgba(217,119,6,0.12)] text-neutral-200 border-warning-600',
        ),
      )
    "
  >
    <Icon name="alert-triangle-filled" size="xs" :class="themeClasses('text-warning-600', 'text-warning-400')" />
    System is currently {{ status }} -
    <a class="hover:underline" href="https://status.systeminit.com" target="_blank"> View status </a>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import { useRealtimeStore } from "@/store/realtime/realtime.store";

const realtimeStore = useRealtimeStore();
const status = computed(() => realtimeStore.applicationStatus);
</script>
