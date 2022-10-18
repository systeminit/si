<template>
  <StatusBarTab :selected="selected">
    <template #icon><Icon name="clock" /></template>
    <template #name>Fix History</template>
    <template #summary></template>
  </StatusBarTab>
</template>

<script lang="ts" setup>
import StatusBarTab from "@/organisms/StatusBar/StatusBarTab.vue";
import Icon from "../../../ui-lib/Icon.vue";

defineProps({
  selected: { type: Boolean, required: true },
});
</script>
