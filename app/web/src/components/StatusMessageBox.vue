<template>
  <div :class="clsx('flex p-xs rounded items-start', getDivClasses)">
    <span class="self-center grow">
      <slot></slot>
    </span>
  </div>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { computed, PropType } from "vue";
import { themeClasses } from "@si/vue-lib/design-system";
import { IconType, Status } from "@/components/StatusIndicatorIcon.vue";

const props = defineProps({
  status: { type: String as PropType<Status> },
  type: { type: String as PropType<IconType>, default: "qualification" },
});

const getDivClasses = computed(() => {
  switch (true) {
    case props.status === "success":
      return themeClasses("text-success-700", "text-success-300");
    case props.status === "warning":
      return themeClasses("text-warning-700", "text-warning-200");
    case props.status === "failure":
      return themeClasses("text-destructive-900", "text-destructive-200");
    default:
      return "";
  }
});
</script>
