<template>
  <Icon
    :name="castIconName(icon?.iconName)"
    :tone="props.tone !== 'inherit' ? (props.tone ?? icon?.tone) : undefined"
    :size="size"
  />
</template>

<script lang="ts">
type Config = Record<string, { iconName: IconNames | "empty"; tone: Tones }>;
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
    ok: { iconName: "check-circle", tone: "success" },
    error: { iconName: "x-circle", tone: "destructive" },
    unknown: { iconName: "question-circle", tone: "warning" },
  },
} as const satisfies Record<string, Config>;

const DEFAULT_ICON = { iconName: "question-circle", tone: "warning" } as const;

export type IconType = keyof typeof CONFIG;
export type IconStatus<T extends IconType> = keyof typeof CONFIG[T];

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
</script>

<!-- eslint-disable vue/component-tags-order,import/first -->
<script lang="ts" setup generic="T extends IconType">
import { computed } from "vue";
import { Icon, IconSizes, IconNames, Tones } from "@si/vue-lib/design-system";

const props = defineProps<{
    type: T;
    status?: IconStatus<T> | null;
    size?: IconSizes;
    tone?: Tones | "inherit";
}>();

const icon = computed(() => {
  return (CONFIG[props.type] as Partial<Config>)[props.status as string ?? "_default"] ?? DEFAULT_ICON;
});
// TODO remove weird cast, it's there because empty is not in IconNames
function castIconName(iconName: IconNames | "empty"): IconNames {
  return icon.value?.iconName as unknown as IconNames;
}
</script>