<template>
  <span class="timestamp">{{ dateStr }}</span>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
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
});

const dateStr = computed(() => {
  return dateString(
    props.date,
    props.size,
    props.relative,
    props.showTimeIfToday,
  );
});
</script>
