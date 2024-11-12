<template>
  <ul>
    <template v-for="item in managementHistory" :key="item.funcRunId">
      <ManagementHistoryCard
        :item="item"
        :selected="item.funcRunId === funcRunId"
        @clickItem="clickItem"
        @history="openHistory"
      />
    </template>
  </ul>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { FuncRunId } from "@/store/func_runs.store";
import { ManagementHistoryItem } from "@/store/management_runs.store";
import ManagementHistoryCard from "./ManagementHistoryCard.vue";

const props = defineProps({
  managementHistory: { type: Array<ManagementHistoryItem> },
  funcRunId: { type: String as PropType<FuncRunId> },
  clickItem: {
    type: Function as PropType<
      (item: ManagementHistoryItem, e: MouseEvent) => void
    >,
    default: undefined,
  },
});

const emit = defineEmits<{
  (e: "history", id: FuncRunId, tabSlug: string): void;
}>();

const openHistory = (id: FuncRunId, slug: string) => {
  emit("history", id, slug);
};
</script>
