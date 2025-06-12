<template>
  <div
    :style="borderStyle(component)"
    :class="
      clsx(
        'flex flex-row items-center rounded-sm border-l-2 p-xs space-x-sm',
        themeClasses('bg-neutral-200', 'bg-neutral-800'),
      )
    "
  >
    <Icon
      :name="getAssetIcon(component.schemaCategory)"
      size="lg"
      class="flex-none"
    />
    <div class="flex-1 min-w-0">
      <TruncateWithTooltip :lineClamp="3" class="text-sm font-semibold">{{
        component.name
      }}</TruncateWithTooltip>
      <TruncateWithTooltip class="text-xs">{{
        component.schemaName
      }}</TruncateWithTooltip>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Icon,
  themeClasses,
  TruncateWithTooltip,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ComponentInList } from "@/workers/types/entity_kind_types";
import { getAssetIcon, getAssetColor } from "./util";

defineProps<{
  component: ComponentInList;
}>();

const borderStyle = (component: ComponentInList) => {
  const color = getAssetColor(component.schemaCategory);
  return `border-color: ${color}`;
};
</script>
