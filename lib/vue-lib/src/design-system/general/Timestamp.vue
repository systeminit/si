<!-- eslint-disable vue/no-v-html -->
<template>
  <span
    v-tooltip="tooltip"
    :class="
      clsx('timestamp', enableDetailTooltip && 'cursor-pointer hover:underline')
    "
    v-html="dateStr"
  ></span>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import clsx from "clsx";
import { TimestampSize, dateString } from "../utils/timestamp";

const props = defineProps({
  date: {
    type: [Date, String] as PropType<string | Date>,
    default: new Date(),
  },
  relative: { type: Boolean, default: false },
  showTimeIfToday: { type: Boolean, default: false },
  size: {
    type: String as PropType<TimestampSize>,
    default: "normal",
  },
  enableDetailTooltip: { type: Boolean },

  // Classes to apply to the date or time text, TODO(Wendy) - not supported for size mini or relative
  dateClasses: { type: String },
  timeClasses: { type: String },
});

const dateStr = computed(() => {
  return dateString(
    props.date,
    props.size,
    props.relative,
    props.showTimeIfToday,
    props.dateClasses,
    props.timeClasses,
  );
});

const tooltip = computed(() => {
  if (!props.enableDetailTooltip) return null;

  return {
    content: props.date,
    delay: { show: 0, hide: 100 },
    instantMove: true,
  };
});
</script>
