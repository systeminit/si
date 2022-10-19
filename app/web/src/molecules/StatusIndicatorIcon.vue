<template>
  <Icon :name="iconName" :tone="iconTone" :size="size" />
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import Icon, { IconNames, IconSizes } from "@/ui-lib/Icon.vue";
import { Tones } from "@/ui-lib/helpers/tones";

// TODO: remove this after refactoring StatusMessageBox
export type Status =
  | "success"
  | "failure"
  | "running"
  | "added"
  | "modified"
  | "deleted";

export type IconType = "change" | "confirmation" | "qualification";

const CONFIG = {
  change: {
    added: { iconName: "plus-circle", tone: "success" },
    deleted: { iconName: "minus-circle", tone: "destructive" },
    modified: { iconName: "tilde-circle", tone: "warning" },
  },
  confirmation: {
    success: { iconName: "check-square", tone: "success" },
    failure: { iconName: "x-square", tone: "destructive" },
    running: { iconName: "loader", tone: "action" },
  },
  qualification: {
    success: { iconName: "check-circle", tone: "success" },
    failure: { iconName: "x-circle", tone: "destructive" },
    running: { iconName: "loader", tone: "action" },
  },
};

// NOTE - would ideally pull in the real types here but generics are not yet supported
// could also think about breaking this into multiple components, but it's nice to keep things consistent
const props = defineProps({
  type: { type: String as PropType<IconType>, required: true },
  status: { type: String },
  size: { type: String as PropType<IconSizes> },
});

// const CONFIG: Record<Status, IconNames> = {
//   success: "check-circle",
//   failure: "x-circle",
//   running: "loader",
//   added: "plus-circle",
//   modified: "edit",
//   deleted: "minus-circle",
// };

const iconName = computed<IconNames>(
  () =>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (CONFIG as any)[props.type]?.[props.status || ""]?.iconName ||
    "question-circle",
);
const iconTone = computed<Tones>(
  () =>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (CONFIG as any)[props.type]?.[props.status || ""]?.tone || "warning",
);
</script>
