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
      iconTone="neutral"
      size="xs"
      @click="emit('toggleExpand')"
    />
    <FlexRender
      v-else
      :render="cell.column.columnDef.cell"
      :props="cell.getContext()"
    />
  </td>
</template>

<script lang="ts" setup>
import { IconButton, themeClasses } from "@si/vue-lib/design-system";
import { FlexRender, Cell } from "@tanstack/vue-table";
import clsx from "clsx";
import { computed, PropType } from "vue";

const props = defineProps({
  cell: {
    type: Object as PropType<
      Cell<
        {
          displayName: string;
          userName: string;
          userId?: string;
          userEmail?: string;
          kind: string;
          timestamp: string;
          entityType: string;
          entityName: string;
          changeSetId?: string;
          changeSetName?: string;
          metadata: Record<string, unknown>;
        },
        unknown
      >
    >,
    required: true,
  },
  rowExpanded: { type: Boolean },
});

const tooltip = computed(() => {
  if (props.cell.column.id === "json") {
    return {
      content: props.rowExpanded ? "Collapse Row" : "Expand Row",
      delay: { show: 0, hide: 100 },
      instantMove: true,
    };
  }

  return null;
});

const emit = defineEmits<{
  (e: "select"): void;
  (e: "toggleExpand"): void;
}>();
</script>
