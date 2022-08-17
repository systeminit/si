<template>
  <template v-if="iconType === 'solid'">
    <SolidCheck
      v-if="status === 'success'"
      class="text-success-500 inline align-middle"
    />
    <SolidExclamation
      v-else-if="status === 'failure'"
      class="text-destructive-500 inline align-middle"
    />
    <SolidPlusCircleIcon
      v-else-if="status === 'added'"
      class="text-success-500 inline align-middle"
    />
    <SpinnerIcon
      v-else-if="status === 'loading'"
      class="text-warning-500 inline align-middle animate-spin"
    />
    <SolidMinusCircleIcon
      v-else-if="status === 'deleted'"
      class="text-destructive-500 inline align-middle"
    />
    <SolidPencilAltIcon
      v-else-if="status === 'modified'"
      class="text-warning-500 inline align-middle"
    />
  </template>

  <template v-else>
    <OutlineCheck
      v-if="status === 'success'"
      class="text-success-500 inline align-middle"
    />
    <OutlinePlusCircleIcon
      v-else-if="status === 'added'"
      class="text-success-500 inline align-middle"
    />
    <OutlineExclamation
      v-else-if="status === 'failure'"
      class="text-destructive-500 inline align-middle"
    />
    <OutlineMinusCircleIcon
      v-else-if="status === 'deleted'"
      class="text-destructive-500 inline align-middle"
    />
    <SpinnerIcon
      v-else-if="status === 'loading'"
      class="text-warning-500 inline align-middle"
    />
    <OutlinePencilAltIcon
      v-else-if="status === 'modified'"
      class="text-warning-500 inline align-middle"
    />
  </template>
</template>

<script lang="ts" setup>
import {
  CheckCircleIcon as OutlineCheck,
  ExclamationIcon as OutlineExclamation,
  MinusCircleIcon as OutlineMinusCircleIcon,
  PencilAltIcon as OutlinePencilAltIcon,
  PlusCircleIcon as OutlinePlusCircleIcon,
} from "@heroicons/vue/outline";
import {
  CheckCircleIcon as SolidCheck,
  ExclamationIcon as SolidExclamation,
  MinusCircleIcon as SolidMinusCircleIcon,
  PencilAltIcon as SolidPencilAltIcon,
  PlusCircleIcon as SolidPlusCircleIcon,
} from "@heroicons/vue/solid";
import SpinnerIcon from "@/atoms/CustomIcons/SpinnerIcon.vue";

export type Status =
  | "success"
  | "failure"
  | "loading"
  | "added"
  | "modified"
  | "deleted";

export type IconType = "solid" | "outline";

withDefaults(
  defineProps<{
    status: Status;
    iconType?: IconType;
  }>(),
  {
    iconType: "solid",
  },
);
</script>
