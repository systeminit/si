<template>
  <StatusBarTab :selected="props.selected">
    <template #icon><Icon class="text-white" name="clock" /></template>
    <template #name>{{
      stats.total > 0 ? "Changes" : "No Changes Yet..."
    }}</template>
    <template #summary>
      <StatusBarTabPill v-if="stats.total > 0">
        <span class="font-bold">Total:&nbsp; {{ stats.total }}</span>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="stats.added > 0"
        class="bg-success-100 text-success-700 font-bold"
      >
        + {{ stats.added }}
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="stats.modified > 0"
        class="bg-warning-100 text-warning-700 font-bold"
      >
        ~ {{ stats.modified }}
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="stats.deleted > 0"
        class="bg-destructive-100 text-destructive-700 font-bold"
      >
        - {{ stats.deleted }}
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script setup lang="ts">
import { computed } from "vue";
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import StatusBarTabPill from "@/organisms/StatusBar/StatusBarTabPill.vue";
import Icon from "@/ui-lib/Icon.vue";
import { useComponentsStore } from "@/store/components.store";

const props = defineProps<{
  selected: boolean;
}>();

const componentsStore = useComponentsStore();
const stats = computed(() => componentsStore.changeStatsSummary);
</script>
