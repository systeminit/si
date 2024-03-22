<template>
  <div :class="classes">
    <Icon :name="icon" size="none" />
  </div>
</template>

<script lang="ts" setup>
import { Icon, IconNames } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { PropType, computed } from "vue";

const props = defineProps({
  overridden: { type: Boolean },
  icon: { type: String as PropType<IconNames>, required: true },
  header: { type: Boolean },
});

const classes = computed(() => {
  if (props.header) {
    return clsx(
      "border dark:group-hover/header:hover:text-action-500 group-hover/header:hover:text-action-300 rounded p-[1px] m-[2px] w-5 h-5 cursor-pointer",
      props.overridden
        ? "text-warning-500 dark:text-warning-300 border-warning-500 dark:border-warning-300 dark:group-hover/header:hover:border-action-500 group-hover/header:hover:border-action-300 dark:group-hover/header:text-shade-100 dark:group-hover/header:border-shade-100 group-hover/header:text-shade-0 group-hover/header:border-shade-0"
        : "border-transparent",
    );
  } else {
    return clsx(
      "border hover:text-action-500 dark:hover:text-action-300 rounded p-[2px] w-6 h-6",
      props.overridden
        ? " text-warning-500 dark:text-warning-300 border-warning-500 dark:border-warning-300 hover:border-action-500 dark:hover:border-action-300"
        : "border-transparent",
    );
  }
});
</script>
