<template>
  <div
    v-if="status !== 'operational' && status !== 'unknown'"
    id="status_container"
  >
    <div
      class="realtime-status-page-state"
      :class="
        clsx({
          'bg-destructive-400': status === 'unavailable',
          'bg-warning-600': status === 'degraded',
          'bg-action-600': status === 'maintenance',
          'bg-success-400': status === 'operational',
        })
      "
    ></div>
    <div class="text-xs realtime-status-page-description">
      Currently {{ status === "maintenance" ? "in maintenance" : status }}, see
      <a
        class="text-action-600"
        href="https://status.systeminit.com"
        target="_blank"
        >Status</a
      >
      for more information
    </div>
    <div id="clear"></div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import clsx from "clsx";
import { useRealtimeStore } from "@/store/realtime/realtime.store";

const realtimeStore = useRealtimeStore();
const status = computed(() => realtimeStore.applicationStatus);
</script>

<style>
.realtime-status-page-state {
  margin-left: 0.5em;
  margin-right: 0.2em;
  width: 5px;
  height: 5px;
  border-radius: 100%;
  float: left;
}
#status_container {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
}
.realtime-status-page-description {
  width: 350px;
  float: left;
  flex: 1;
}
.clear {
  clear: both;
}
</style>
