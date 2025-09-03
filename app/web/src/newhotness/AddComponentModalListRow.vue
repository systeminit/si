<template>
  <div
    :style="{ 'border-left-color': rowData.color }"
    :class="
      clsx(
        'absolute top-0 left-0 w-full flex flex-row items-center px-xs gap-2xs border-l-[3px] cursor-pointer',
        'hover:text-action-300 bg-neutral-700',
      )
    "
  >
    <Icon
      v-if="rowData.type === 'category'"
      size="lg"
      :name="open ? 'chevron--down' : 'chevron--right'"
    />

    <Icon
      v-if="rowData.type === 'category' && rowData.icon"
      size="md"
      :name="rowData.icon"
      class="mr-xs"
    />
    {{ rowData.name }}
  </div>
</template>

<script setup lang="ts">
import clsx from "clsx";
import { Icon, IconNames } from "@si/vue-lib/design-system";

defineProps<{
  rowData: AddComponentRowData;
  open?: boolean;
}>();
</script>

<script lang="ts">
interface AddComponentBaseRowData {
  color: string;
  name: string;
}

export type AddComponentRowData = AddComponentBaseRowData &
  ({ type: "category"; icon?: IconNames } | { type: "schema" });
</script>
