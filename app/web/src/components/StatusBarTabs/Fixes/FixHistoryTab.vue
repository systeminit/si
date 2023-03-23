<template>
  <StatusBarTab :selected="selected">
    <template #icon><Icon name="clock" /></template>
    <template #name>Apply History</template>
    <template #summary></template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import StatusBarTab from "@/components/StatusBar/StatusBarTab.vue";

defineProps({
  selected: { type: Boolean, required: true },
});
</script>
