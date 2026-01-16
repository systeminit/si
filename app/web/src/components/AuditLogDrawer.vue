<template>
  <tr
    v-show="expanded"
    :class="
      clsx(
        'border',
        themeClasses('border-neutral-300', 'border-neutral-900'),
        Number(row.id) % 2 === 0
          ? themeClasses(' bg-neutral-200', 'bg-neutral-700')
          : themeClasses(' bg-neutral-100', 'bg-neutral-800'),
      )
    "
  >
    <td :colspan="colspan" class="p-0">
      <CodeViewer showTitle title="Raw Event Data" :code="JSON.stringify(row.original, null, 2)" />
    </td>
  </tr>
</template>

<script setup lang="ts">
import { themeClasses } from "@si/vue-lib/design-system";
import { Row } from "@tanstack/vue-table";
import { PropType } from "vue";
import clsx from "clsx";
import { AuditLogDisplay } from "@/store/logs.store";
import CodeViewer from "./CodeViewer.vue";

defineProps({
  row: {
    type: Object as PropType<Row<AuditLogDisplay>>,
    required: true,
  },
  colspan: {
    type: Number,
    required: true,
  },
  expanded: {
    type: Boolean,
  },
});
</script>
