<template>
  <StatusBarTab :selected="selected">
    <template #icon>
      <StatusIndicatorIcon type="qualification" :status="overallStatus" />
    </template>
    <template #name>
      Qualifications
      <!-- <template v-if="overallStatus === 'running'"> Running... </template>
      <template v-else-if="componentStats.total > 0"> Qualifications </template>
      <template v-else>No Qualifications Run...</template> -->
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
        <StatusIndicatorIcon type="qualification" status="success" size="xs" />
        <div>
          {{ componentStats.success }}
        </div>
      </StatusBarTabPill>

      <StatusBarTabPill
        v-if="componentStats.warning"
        class="bg-warning-100 text-warning-600 font-bold"
      >
        <StatusIndicatorIcon type="qualification" status="warning" size="xs" />
        <div>
          {{ componentStats.warning }}
        </div>
      </StatusBarTabPill>

      <StatusBarTabPill
        v-if="componentStats.failure"
        class="bg-destructive-100 text-destructive-600 font-bold"
      >
        <StatusIndicatorIcon type="qualification" status="failure" size="xs" />
        <div>
          {{ componentStats.failure }}
        </div>
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import { useQualificationsStore } from "@/store/qualifications.store";
import StatusBarTabPill from "./StatusBarTabPill.vue";
import StatusBarTab from "./StatusBarTab.vue";

const props = defineProps({
  selected: Boolean,
});

const qualificationStore = useQualificationsStore();
const componentStats = computed(() => qualificationStore.componentStats);
const overallStatus = computed(() => qualificationStore.overallStatus);
</script>
