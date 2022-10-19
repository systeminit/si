<template>
  <StatusBarTab :selected="selected">
    <template #icon>
      <StatusIndicatorIcon type="confirmation" :status="workspaceStatus" />
    </template>
    <template #name>
      Confirmations
      <!-- <template v-if="workspaceStatus === 'running'">Running...</template>
      <template v-else-if="componentStats.total > 0">Confirmations</template>
      <template v-else>No Confirmations Run...</template> -->
    </template>
    <template v-if="componentStats.total" #summary>
      <StatusBarTabPill v-if="componentStats.total" class="border-white">
        Total:
        <b class="ml-1">{{ componentStats.total }}</b>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="componentStats.success"
        class="bg-success-100 text-success-600 font-bold"
      >
        <StatusIndicatorIcon type="confirmation" status="success" size="xs" />
        <div class="pl-px">
          {{ componentStats.success }}
        </div>
      </StatusBarTabPill>

      <StatusBarTabPill
        v-if="componentStats.failure"
        class="bg-destructive-100 text-destructive-600 font-bold"
      >
        <StatusIndicatorIcon type="confirmation" status="failure" size="xs" />
        <div class="pl-px">
          {{ componentStats.failure }}
        </div>
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusIndicatorIcon from "@/molecules/StatusIndicatorIcon.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import { useResourcesStore } from "@/store/resources.store";

defineProps<{ selected: boolean }>();

const resourcesStore = useResourcesStore();
const componentStats = computed(
  () => resourcesStore.componentsConfirmationStats,
);

const workspaceStatus = computed(
  () => resourcesStore.workspaceConfirmationResult,
);
</script>
