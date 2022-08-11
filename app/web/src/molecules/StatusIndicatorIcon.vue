<template>
  <template v-if="iconType === 'solid'">
    <SolidCheck
      v-if="status === 'success'"
      class="text-success-500 inline align-middle"
    />
    <SolidX
      v-else-if="status === 'failure'"
      class="text-destructive-500 inline align-middle"
    />
    <SolidClock
      v-else-if="status === 'loading'"
      class="text-warning-500 inline align-middle"
    />
  </template>

  <template v-else>
    <OutlineCheck
      v-if="status === 'success'"
      class="text-success-500 inline align-middle"
    />
    <OutlineX
      v-else-if="status === 'failure'"
      class="text-destructive-500 inline align-middle"
    />
    <OutlineClock
      v-else-if="status === 'loading'"
      class="text-warning-500 inline align-middle"
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
    iconType?: IconType;
  }>(),
  {
    iconType: "outline",
  },
);
</script>
