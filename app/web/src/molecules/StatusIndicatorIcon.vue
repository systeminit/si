<template>
  <Icon :name="iconName" :class="colorClass" size="full" />
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import Icon, { IconNames } from "@/ui-lib/Icon.vue";

export type Status =
  | "success"
  | "failure"
  | "running"
  | "added"
  | "modified"
  | "deleted";

const props = defineProps({
  status: { type: String as PropType<Status>, required: true },
});

const ICON_NAME_MAP: Record<Status, IconNames> = {
  success: "check-circle",
  failure: "x-circle",
  running: "loader",
  added: "plus-circle",
  modified: "edit",
  deleted: "minus-circle",
};

const iconName = computed(() => ICON_NAME_MAP[props.status]);

const colorClass = computed(() => {
  return {
    success: "text-success-500",
    failure: "text-destructive-500",
    running: "text-info-500",
    added: "text-success-500",
    modified: "text-warning-500",
    deleted: "text-destructive-500",
  }[props.status];
});
</script>
