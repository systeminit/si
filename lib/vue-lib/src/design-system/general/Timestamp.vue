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
import { computed, onMounted, onUnmounted, PropType, ref } from "vue";
import clsx from "clsx";
import { TimestampSize, dateString } from "../utils/timestamp";

const props = defineProps({
  date: {
    type: [Date, String] as PropType<string | Date>,
    default: new Date(),
  },
  relative: {
    type: String as PropType<"standard" | "shorthand" | "disabled">,
    default: "disabled",
  },
  showTimeIfToday: { type: Boolean, default: false },
  size: {
    type: String as PropType<TimestampSize>,
    default: "normal",
  },
  enableDetailTooltip: { type: Boolean },
  refresh: { type: Boolean, default: false },

  // Classes to apply to the date or time text, TODO(Wendy) - not supported for size mini or relative
  dateClasses: { type: String },
  timeClasses: { type: String },
});

const trigger = ref(false);

// TODO(nick): make this more elegant. Right now, we just want to re-compute based on the trigger.
// This is an anti-pattern and I am sorry... kinda.
const dateStr = computed(() => {
  if (trigger.value) {
    return dateString(
      props.date,
      props.size,
      props.relative,
      props.showTimeIfToday,
      props.dateClasses,
      props.timeClasses,
    );
  }
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

  const content = String(props.date);

  return {
    content,
    theme: "instant-show",
  };
});

// TODO(nick): align the refresh interval with the wall clock rather than doing it every 60
// seconds. That way, the "17 minutes ago" stamp will be more precise for the user.
if (props.refresh) {
  onMounted(() => {
    const interval = setInterval(() => {
      trigger.value = !trigger.value;
    }, 60000);

    onUnmounted(() => {
      clearInterval(interval);
    });
  });
}
</script>
