<template>
  <StatusBarTab :selected="props.selected">
    <template #icon>
      <Icon class="text-white" name="tilde-square" />
    </template>
    <template #name>Diff</template>
    <template #summary>
      <StatusBarTabPill v-if="stats.total > 0">
        <span class="font-bold">Total:&nbsp; {{ stats.total }}</span>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="stats.added > 0"
        class="bg-success-100 text-success-700 font-bold"
      >
        <StatusIndicatorIcon type="change" status="added" size="xs" />
        <div>{{ stats.added }}</div>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="stats.modified > 0"
        class="bg-warning-100 text-warning-700 font-bold"
      >
        <StatusIndicatorIcon type="change" status="modified" size="xs" />
        <div>{{ stats.modified }}</div>
      </StatusBarTabPill>
      <StatusBarTabPill
        v-if="stats.deleted > 0"
        class="bg-destructive-100 text-destructive-700 font-bold"
      >
        <StatusIndicatorIcon type="change" status="deleted" size="xs" />
        <div>{{ stats.deleted }}</div>
      </StatusBarTabPill>
    </template>
  </StatusBarTab>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { Icon } from "@si/vue-lib/design-system";
import { useComponentsStore } from "@/store/components.store";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import StatusBarTabPill from "./StatusBarTabPill.vue";
import StatusBarTab from "./StatusBarTab.vue";

const props = defineProps({
  selected: Boolean,
});

const componentsStore = useComponentsStore();
const stats = computed(() => componentsStore.changeStatsSummary);
</script>
