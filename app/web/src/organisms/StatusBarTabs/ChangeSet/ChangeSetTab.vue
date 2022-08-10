<template>
  <StatusBarTabPill>
    Total:
    <span class="font-bold"
      >&nbsp;
      {{
        stats.added.length + stats.modified.length + stats.deleted.length
      }}</span
    >
  </StatusBarTabPill>
  <StatusBarTabPill class="bg-success-100 text-success-500 font-bold">
    + {{ stats.added.length }}
  </StatusBarTabPill>
  <StatusBarTabPill class="bg-warning-100 text-warning-500 font-bold">
    ~ {{ stats.modified.length }}
  </StatusBarTabPill>
  <StatusBarTabPill class="bg-destructive-100 text-destructive-500 font-bold">
    - {{ stats.deleted.length }}
  </StatusBarTabPill>
</template>

<script setup lang="ts">
import { ComponentStats } from "@/api/sdf/dal/change_set";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { ref } from "vue";
import { untilUnmounted } from "vuse-rx";

const stats = ref<ComponentStats>({
  added: [],
  deleted: [],
  modified: [],
});

untilUnmounted(ChangeSetService.getStats()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
  } else {
    stats.value = response.componentStats;
  }
});
</script>
