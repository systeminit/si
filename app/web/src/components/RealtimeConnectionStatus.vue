<template>
  <div
    class="realtime-connection-status"
    :class="
      clsx({
        'bg-success-400': status === 'connected',
        'bg-error-400': status === 'disconnected',
        'bg-neutral-400': status === 'closing' || status === 'closed',
      })
    "
  ></div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import { useRealtimeStore } from "@/store/realtime/realtime.store";

const realtimeStore = useRealtimeStore();
const status = computed(() => realtimeStore.connectionStatus);
</script>

<style>
.realtime-connection-status {
  width: 3px;
  height: 3px;
  border-radius: 100%;
  position: absolute;
  bottom: 5px;
  left: 5px;
  z-index: 9999;
}
</style>
