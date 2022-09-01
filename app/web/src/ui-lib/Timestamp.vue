<template>
  <span class="timestamp">{{ dateStr }}</span>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import { format, formatDistanceToNowStrict } from "date-fns";

export type TimestampSize = "mini" | "normal" | "extended";

const props = defineProps({
  date: { type: Date, default: new Date() },
  relative: { type: Boolean, default: false },
  size: {
    type: String as PropType<TimestampSize>,
    default: "normal",
  },
});

const dateStr = computed(() => {
  const d = props.date;
  if (props.size === "mini") {
    if (props.relative) {
      return formatDistanceToNowStrict(d);
    }
    return format(d, "M/d/y");
  } else if (props.size === "extended") {
    if (props.relative) {
      return formatDistanceToNowStrict(d);
    }
    return `${format(d, "eeee MMMM do, y")} at ${format(d, "h:mm:ss a")}`;
  }
  if (props.relative) {
    return formatDistanceToNowStrict(d);
  }
  return format(d, "MMMM d, y");
});
</script>
