<template>
  <Icon :name="iconName" :tone="iconTone" :size="size" />
</template>

<script lang="ts">
const CONFIG = {
  change: {
    added: { iconName: "plus-circle", tone: "success" },
    deleted: { iconName: "x", tone: "destructive" },
    modified: { iconName: "component-changes-large", tone: "warning" },
    unmodified: { iconName: "empty-square", tone: "empty" },
  },
  confirmation: {
    success: { iconName: "component-qualified-large", tone: "success" },
    failure: { iconName: "component-not-qualified-large", tone: "destructive" },
    running: { iconName: "loader", tone: "action" },
    _default: { iconName: "check-circle", tone: "success" },
  },
  qualification: {
    success: { iconName: "component-qualified-large", tone: "success" },
    warning: { iconName: "component-not-qualified-large", tone: "warning" },
    failure: { iconName: "component-not-qualified-large", tone: "destructive" },
    running: { iconName: "loader", tone: "action" },
    notexists: { iconName: "empty-square", tone: "empty" },
  },
  fix: {
    success: { iconName: "check2", tone: "success" },
    failure: { iconName: "alert-triangle", tone: "destructive" },
    unstarted: { iconName: "loader", tone: "neutral" },
    running: { iconName: "loader", tone: "action" },
  },
  resource: {
    exists: { iconName: "resource-passed-large", tone: "success" },
    notexists: { iconName: "empty-square", tone: "empty" },
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
  | "show";

// NOTE - would ideally pull in the real types here but generics are not yet supported
// could also think about breaking this into multiple components, but it's nice to keep things consistent
const props = defineProps({
  type: { type: String as PropType<IconType>, required: true },
  status: { type: String },
  size: { type: String as PropType<IconSizes> },
  tone: {
    type: String as PropType<Tones>,
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
