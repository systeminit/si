<template>
  <template v-if="iconType === 'solid'">
    <SolidCheck
      v-if="status === 'success'"
      :class="iconClasses"
      class="text-success-500"
    />
    <SolidX
      v-else-if="status === 'failure'"
      :class="iconClasses"
      class="text-destructive-500"
    />
    <SolidClock
      v-else-if="status === 'loading'"
      :class="iconClasses"
      class="text-warning-500"
    />
  </template>

  <template v-else>
    <OutlineCheck
      v-if="status === 'success'"
      :class="iconClasses"
      class="text-success-500"
    />
    <OutlineX
      v-else-if="status === 'failure'"
      :class="iconClasses"
      class="text-destructive-500"
    />
    <OutlineClock
      v-else-if="status === 'loading'"
      :class="iconClasses"
      class="text-warning-500"
    />
  </template>
</template>

<script lang="ts" setup>
import {
  CheckCircleIcon as OutlineCheck,
  ClockIcon as OutlineClock,
  XCircleIcon as OutlineX,
} from "@heroicons/vue/outline";
import {
  CheckCircleIcon as SolidCheck,
  ClockIcon as SolidClock,
  XCircleIcon as SolidX,
} from "@heroicons/vue/solid";
import { defineProps } from "vue";

export type Status = "success" | "failure" | "loading";

export type IconType = "solid" | "outline";

withDefaults(
  defineProps<{
    status: Status;
    iconType: IconType;
  }>(),
  {
    iconType: "outline",
  },
);

const iconClasses = "inline align-middle";
</script>
