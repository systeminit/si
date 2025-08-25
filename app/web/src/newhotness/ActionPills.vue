<template>
  <div
    :class="
      clsx(
        'gap-1 items-center',
        mode === 'grid' && 'grid grid-cols-5',
        mode === 'row' && 'flex flex-row',
      )
    "
  >
    <TextPill
      v-if="
        showNoPendingActions &&
        (!actionCounts || Object.keys(actionCounts).length === 0)
      "
      variant="key2"
      size="sm"
      class="text-xs flex items-center gap-1"
    >
      No pending actions
    </TextPill>
    <TextPill
      v-for="(actionData, actionName) in actionCounts"
      :key="actionName"
      v-tooltip="getTooltip(actionName, actionData.count, actionData.hasFailed)"
      variant="key2"
      size="sm"
      class="text-xs flex items-center gap-1"
    >
      <Icon
        :name="getIcon(actionName)"
        :class="
          actionData.hasFailed ? 'text-destructive-500' : 'text-neutral-500'
        "
        size="xs"
      />
      {{ actionData.count }}
    </TextPill>
  </div>
</template>

<script lang="ts" setup>
import { Icon, IconNames, TextPill } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps<{
  actionCounts?: Record<string, { count: number; hasFailed: boolean }>;
  mode: "grid" | "row";
  showNoPendingActions?: boolean;
}>();

const getIcon = (actionName: string): IconNames => {
  const iconMap: Record<string, IconNames> = {
    Create: "plus",
    Update: "tilde",
    Refresh: "refresh",
    Destroy: "trash",
    Delete: "trash",
    Manual: "play",
  };
  return iconMap[actionName] || "play";
};

const getTooltip = (actionName: string, count: number, hasFailed: boolean) => {
  const actionWord = actionName.toLowerCase();
  const plural = count > 1 ? "s" : "";

  if (hasFailed && count === 1) {
    return `1 pending ${actionWord} action failed`;
  } else if (hasFailed) {
    return `${count} pending ${actionWord} action${plural} (including failed)`;
  } else {
    return `${count} pending ${actionWord} action${plural}`;
  }
};
</script>
