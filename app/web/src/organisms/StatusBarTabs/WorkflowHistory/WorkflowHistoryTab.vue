<template>
  <StatusBarTab :selected="selected">
    <template #icon><WorkflowStatusIcon status="success" /></template>
    <!-- TODO(wendy) - replace fixed icon with updating one -->
    <template #name>Workflow History</template>
    <template #summary></template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import WorkflowStatusIcon from "@/molecules/WorkflowStatusIcon.vue";

defineProps({
  selected: { type: Boolean, required: true },
});
</script>
