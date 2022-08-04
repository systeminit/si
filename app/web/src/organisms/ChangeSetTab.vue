<template>
  <StatusBarTabPill>
    Total: <span class="font-bold">&nbsp; {{ total }}</span>
  </StatusBarTabPill>
  <StatusBarTabPill class="bg-success-100 text-success-500 font-bold">
    + {{ stats.added }}
  </StatusBarTabPill>
  <StatusBarTabPill class="bg-warning-100 text-warning-500 font-bold">
    ~ {{ stats.modified }}
  </StatusBarTabPill>
  <StatusBarTabPill class="bg-destructive-100 text-destructive-500 font-bold">
    - {{ stats.deleted }}
  </StatusBarTabPill>
</template>

<script setup lang="ts">
import { ComponentStats } from "@/api/sdf/dal/change_set";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { ref } from "vue";
import { untilUnmounted } from "vuse-rx";

const defaultComponentStats: ComponentStats = {
  added: 0,
  deleted: 0,
  modified: 0,
};

const stats = ref<ComponentStats>(defaultComponentStats);
const total = ref<number>(0);

untilUnmounted(ChangeSetService.getStats()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
  } else {
    stats.value = response.componentStats;
    total.value =
      response.componentStats.added +
      response.componentStats.deleted +
      response.componentStats.modified;
  }
});
</script>
