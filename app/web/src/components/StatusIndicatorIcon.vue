<template>
  <Icon
    :name="iconName"
    :tone="tone !== 'inherit' ? iconTone : undefined"
    :size="size"
  />
</template>

<script lang="ts">
const CONFIG = {
  change: {
    added: { iconName: "plus-square", tone: "success" },
    deleted: { iconName: "minus-square", tone: "destructive" },
    modified: { iconName: "tilde-square", tone: "warning" },
    unmodified: { iconName: "empty", tone: "empty" },
  },
  qualification: {
    success: { iconName: "check-hex-outline", tone: "success" },
    warning: { iconName: "x-hex-outline", tone: "warning" },
    failure: { iconName: "x-hex-outline", tone: "destructive" },
    running: { iconName: "loader", tone: "action" },
    notexists: { iconName: "empty", tone: "empty" },
  },
  fix: {
    success: { iconName: "check2", tone: "success" },
    failure: { iconName: "alert-triangle", tone: "destructive" },
    unstarted: { iconName: "loader", tone: "neutral" },
    running: { iconName: "loader", tone: "action" },
  },
  resource: {
    exists: { iconName: "check-hex", tone: "success" },
    notexists: { iconName: "x-hex", tone: "destructive" },
  },
  action: {
    create: { iconName: "resource-create", tone: "success" },
    delete: { iconName: "resource-delete", tone: "destructive" },
    refresh: { iconName: "resource-refresh", tone: "action" },
    other: { iconName: "resource-question", tone: "action" },
  },
  actions: {
    show: { iconName: "chevron--right", tone: "neutral" },
  },
  funcTest: {
    success: { iconName: "check-circle", tone: "success" },
    running: { iconName: "loader", tone: "action" },
    failure: { iconName: "alert-triangle", tone: "destructive" },
    error: { iconName: "alert-triangle", tone: "destructive" },
    unknown: { iconName: "question-circle", tone: "warning" },
    _default: { iconName: "help-circle", tone: "warning" },
  },
};

export type IconType = keyof typeof CONFIG;
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { computed, PropType } from "vue";
import { Icon, IconSizes, IconNames, Tones } from "@si/vue-lib/design-system";

// TODO: remove this after refactoring StatusMessageBox
// TODO(nick,paulo,paul,wendy): remove "neverStarted" once the fix flow is working again.
export type Status =
  | "success"
  | "failure"
  | "unknown"
  | "warning"
  | "running"
  | "added"
  | "modified"
  | "neverStarted"
  | "unmodified"
  | "deleted"
  | "show"
  | "pending"
  | "error";

// NOTE - would ideally pull in the real types here but generics are not yet supported
// could also think about breaking this into multiple components, but it's nice to keep things consistent
const props = defineProps({
  type: { type: String as PropType<IconType>, required: true },
  status: { type: String },
  size: { type: String as PropType<IconSizes> },
  tone: {
    type: String as PropType<Tones | "inherit">,
    required: false,
  },
});

const iconName = computed<IconNames>(
  () =>
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (CONFIG as any)[props.type]?.[props.status || "_default"]?.iconName ||
    "question-circle",
);
const iconTone = computed<Tones>(
  () =>
    props.tone ||
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (CONFIG as any)[props.type]?.[props.status || "_default"]?.tone ||
    "warning",
);
</script>
