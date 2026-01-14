<template>
  <td
    v-tooltip="tooltip"
    align="center"
    :class="
      clsx(
        'border-x border-collapse',
        cell.column.id === 'json' && 'w-8',
        themeClasses('border-neutral-300', 'border-neutral-900'),
      )
    "
  >
    <IconButton
      v-if="cell.column.id === 'json'"
      :icon="rowExpanded ? 'collapse-row' : 'expand-row'"
      iconIdleTone="neutral"
      size="xs"
      @click="emit('toggleExpand')"
    />
    <span v-else-if="cellDataTruncated" v-tooltip="{ content: cellRawData }">
      <FlexRender
        :render="cell.column.columnDef.cell"
        :props="{ ...cell.getContext(), getValue: () => cellDataTruncated }"
      />
    </span>
    <FlexRender v-else :render="cell.column.columnDef.cell" :props="cell.getContext()" />
  </td>
</template>

<script lang="ts" setup>
import { IconButton, themeClasses } from "@si/vue-lib/design-system";
import { FlexRender, Cell } from "@tanstack/vue-table";
import clsx from "clsx";
import { computed, PropType } from "vue";
import { AuditLogDisplay } from "@/store/logs.store";

const MAX_STRING_LENGTH = 100;

const props = defineProps({
  cell: {
    type: Object as PropType<Cell<AuditLogDisplay, unknown>>,
    required: true,
  },
  rowExpanded: { type: Boolean },
});

const tooltip = computed(() => {
  if (props.cell.column.id === "json") {
    return {
      content: props.rowExpanded ? "Collapse Row" : "Expand Row",
      theme: "instant-show",
    };
  }

  return null;
});

const cellRawData = computed(() => props.cell.getContext().getValue() as string);

const cellDataTruncated = computed(() => {
  if (cellRawData.value.length > MAX_STRING_LENGTH) {
    return `${cellRawData.value.substring(0, MAX_STRING_LENGTH)}...`;
  }
  return undefined;
});

const emit = defineEmits<{
  (e: "select"): void;
  (e: "toggleExpand"): void;
}>();
</script>
