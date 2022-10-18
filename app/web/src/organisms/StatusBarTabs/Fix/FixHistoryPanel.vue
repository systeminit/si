<template>
  <div class="w-full h-full flex flex-row">
    <div class="w-72 shrink-0 border-shade-100 h-full flex flex-col">
      <span
        class="h-11 border-b border-shade-100 text-lg px-4 flex items-center flex-none"
      >
        Fix Runs
      </span>

      <!-- Sort button and its dropdown -->
      <SiBarButton
        class="h-11 border-b border-shade-100 flex-none"
        tooltip-text="Sort"
        fill-entire-width
      >
        <template #default="{ hovered, open }">
          <div class="flex flex-row items-center">
            {{ selectedSort.title }}
            <SiArrow :nudge="hovered || open" class="ml-1 w-4" />
          </div>
        </template>

        <template #dropdownContent>
          <SiDropdownItem
            v-for="option of sortOptions"
            :key="option.value"
            :checked="selectedSort.value === option.value"
            @select="emit('sort', option)"
          >
            {{ option.title }}
          </SiDropdownItem>
        </template>
      </SiBarButton>

      <!-- List of fixes -->
      <div class="overflow-y-auto flex-expand">
        <div
          v-for="fixBatch in fixListDisplay"
          :key="fixBatch.id"
          :class="
            fixBatch.id === selectedFixBatchId
              ? 'bg-action-500'
              : 'hover:bg-black'
          "
          class="py-2 pl-4 pr-3 cursor-pointer flex flex-row items-center leading-tight"
          @click="selectFixBatch(fixBatch.id)"
        >
          <span class="truncate mr-3 whitespace-nowrap">
            <Timestamp :date="fixBatch.timestamp" size="long" />
          </span>
        </div>
      </div>
    </div>
    <!-- Currently selected FixBatch info panel -->
    <div v-if="selectedFixBatchInfo" class="grow flex flex-row overflow-hidden">
      <div class="w-72 shrink-0 border-shade-100 border-l h-full flex flex-col">
        <span
          class="h-11 border-b border-shade-100 text-lg px-4 flex items-center flex-none"
        >
          Fixes
        </span>
        <div
          v-for="fix in selectedFixBatchInfo.fixes"
          :key="fix.id"
          :class="fix.id === selectedFixId ? 'bg-action-500' : 'hover:bg-black'"
          class="py-2 pl-4 pr-3 cursor-pointer flex flex-row items-center leading-tight"
          @click="selectFix(fix.id)"
        >
          <FixStatusIcon :status="fix.status" size="lg" />
          <span class="truncate mr-3 whitespace-nowrap">
            {{ fix.name }}
          </span>
        </div>
      </div>
      <div v-if="selectedFixInfo" class="bg-shade-100 grow p-4">
        <CodeViewer :code="selectedFixInfo.output">
          <template #title>{{ selectedFixInfo.name }}</template>
        </CodeViewer>
      </div>
      <div
        v-else
        class="grow flex flex-row overflow-hidden bg-shade-100 items-center text-center"
      >
        <p class="w-full text-3xl text-neutral-500">No Fix Selected</p>
      </div>
    </div>
    <div
      v-else
      class="grow flex flex-row overflow-hidden bg-shade-100 items-center text-center"
    >
      <p class="w-full text-3xl text-neutral-500">No Fix Run Selected</p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import SiBarButton from "@/molecules/SiBarButton.vue";
import SiDropdownItem from "@/atoms/SiDropdownItem.vue";
import SiArrow from "@/atoms/SiArrow.vue";
import Timestamp from "@/ui-lib/Timestamp.vue";
import { useFixesStore } from "@/store/fixes/fixes.store";
import CodeViewer from "@/organisms/CodeViewer.vue";
import FixStatusIcon from "@/molecules/FixStatusIcon.vue";

export interface SortOption {
  value: string;
  title: string;
}

const props = defineProps<{
  sortOptions?: SortOption[];
  selectedSort?: SortOption;
}>();

const selectedFixBatchId = ref<number | null>(null);

const selectedFixId = ref<number | null>(null);

const emit = defineEmits<{
  (e: "sort", sortOption: SortOption): void;
}>();

const defaultSortOption = {
  value: "r",
  title: "Newest",
};

const selectedSort = computed(() => {
  return props.selectedSort ?? defaultSortOption;
});

const selectFixBatch = (id: number) => {
  selectedFixBatchId.value = id;
  selectedFixId.value = null;
};

const selectFix = (id: number) => {
  selectedFixId.value = id;
};

const fixesStore = useFixesStore();

const fixBatchesWithFixes = computed(() =>
  fixesStore.allFixBatches.map((batch) => ({
    ...batch,
    fixes: fixesStore
      .fixesOnBatch(batch.id)
      .filter((fix) => fix.status === "success"),
  })),
);

const fixListDisplay = computed(() => {
  if (selectedSort.value.value === "r") {
    return [...fixBatchesWithFixes.value].reverse();
  } else return fixBatchesWithFixes.value;
});

const selectedFixBatchInfo = computed(() => {
  return fixBatchesWithFixes.value.find(
    (fixBatch) => fixBatch.id === selectedFixBatchId.value,
  );
});

const selectedFixInfo = computed(() => {
  if (selectedFixBatchInfo.value) {
    return selectedFixBatchInfo.value.fixes.find(
      (fix) => fix.id === selectedFixId.value,
    );
  }
  return null;
});
</script>
