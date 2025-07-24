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
  "action-state": {
    Dispatched: { iconName: "loader", tone: "action" },
    Failed: { iconName: "alert-triangle", tone: "destructive" },
    OnHold: { iconName: "loader", tone: "neutral" },
    Queued: { iconName: "empty", tone: "empty" },
    Running: { iconName: "loader", tone: "action" },
  },
  "action-runner": {
    success: { iconName: "check2", tone: "success" },
    error: { iconName: "alert-triangle", tone: "destructive" },
    failure: { iconName: "alert-triangle", tone: "destructive" },
    unstarted: { iconName: "loader", tone: "neutral" },
    running: { iconName: "loader", tone: "action" },
  },
  resource: {
    exists: { iconName: "check-hex", tone: "success" },
    notexists: { iconName: "x-hex", tone: "destructive" },
  },
  action: {
    create: { iconName: "plus-hex-outline", tone: "success" },
    Create: { iconName: "plus", tone: "success" },
    delete: { iconName: "minus-hex-outline", tone: "destructive" },
    Destroy: { iconName: "trash", tone: "destructive" },
    refresh: { iconName: "refresh-hex-outline", tone: "action" },
    Refresh: { iconName: "refresh", tone: "action" },
    other: { iconName: "tilde-hex-outline", tone: "action" },
    Manual: { iconName: "play", tone: "action" },
    Update: { iconName: "tilde", tone: "warning" },
  },
  actions: {
    show: { iconName: "chevron--right", tone: "neutral" },
  },
  upgradable: {
    _default: { iconName: "bolt", tone: "action" },
  },
  funcTest: {
    success: { iconName: "check-circle", tone: "success" },
    running: { iconName: "loader", tone: "action" },
    failure: { iconName: "alert-triangle", tone: "destructive" },
    error: { iconName: "alert-triangle", tone: "destructive" },
    unknown: { iconName: "question-circle", tone: "warning" },
    _default: { iconName: "question-circle", tone: "warning" },
  },
  management: {
    Created: { iconName: "loader", tone: "neutral" },
    Dispatched: { iconName: "loader", tone: "neutral" },
    Running: { iconName: "loader", tone: "action" },
    Postprocessing: { iconName: "loader", tone: "action" },
    Failure: { iconName: "close-outline", tone: "destructive" },
    ActionFailure: { iconName: "close-outline", tone: "destructive" },
    Success: { iconName: "checkmark-outline", tone: "success" },
  },
};

export type IconType = keyof typeof CONFIG;
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup>
import { computed } from "vue";
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
const props = defineProps<{
  type: keyof typeof CONFIG;
  status?: string | null;
  size?: IconSizes;
  tone?: Tones | "inherit";
}>();

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
