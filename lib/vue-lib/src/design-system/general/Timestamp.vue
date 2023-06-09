<template>
  <span class="timestamp">{{ dateStr }}</span>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import { format, formatDistanceToNow, parseISO } from "date-fns";
import TimeAgo from "javascript-time-ago";
import en from "javascript-time-ago/locale/en";

export type TimestampSize = "mini" | "normal" | "long" | "extended";

const props = defineProps({
  date: { type: [Date, String], default: new Date() },
  relative: { type: Boolean, default: false },
  showTimeIfToday: { type: Boolean, default: false },
  size: {
    type: String as PropType<TimestampSize>,
    default: "normal",
  },
});

TimeAgo.addLocale(en);
const timeAgo = new TimeAgo("en-US");

const dateStr = computed(() => {
  let d: Date;
  if (_.isString(props.date)) {
    d = parseISO(props.date);
  } else {
    d = props.date;
  }

  if (
    !props.relative &&
    props.showTimeIfToday &&
    d.toDateString() === new Date().toDateString()
  ) {
    if (props.size === "long" || props.size === "extended") {
      return `Today at ${format(d, "h:mm:ss a")}`;
    }
    return format(d, "h:mm:ss a");
  }

  if (props.size === "mini") {
    if (props.relative) {
      return timeAgo.format(d, "mini-minute-now");
    }
    return format(d, "M/d/y");
  } else if (props.size === "extended") {
    if (props.relative) {
      return `${formatDistanceToNow(d)} ago`;
    }
    return `${format(d, "eeee MMMM do, y")} at ${format(d, "h:mm:ss a")}`;
  } else if (props.size === "long") {
    if (props.relative) {
      return `${formatDistanceToNow(d)} ago`;
    }
    return `${format(d, "M/d/y")} at ${format(d, "h:mm:ss a")}`;
  }
  if (props.relative) {
    return timeAgo.format(d);
  }
  return format(d, "MMMM d, y");
});
</script>
