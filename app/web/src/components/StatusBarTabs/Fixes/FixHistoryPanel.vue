<template>
  <div class="w-full h-full flex flex-row">
    <div
      v-if="fixListDisplay.length > 0"
      class="w-72 shrink-0 border-shade-100 h-full flex flex-col"
    >
      <span
        class="h-11 border-b border-shade-100 text-lg px-4 flex items-center flex-none"
      >
        Fix Runs
      </span>

      <!-- Sort button and its dropdown -->
      <!-- FIXME(nick): restore filter and dropdown -->

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
          <div
            class="truncate mr-3 whitespace-nowrap flex flex-row gap-2 items-center"
          >
            <StatusIndicatorIcon
              v-if="fixBatch.status"
              type="fix"
              :status="fixBatch.status"
            />
            <div class="flex flex-col">
              <div class="text-sm font-bold truncate">
                {{ fixBatch.author }}
              </div>
              <div class="text-xs italic">
                <Timestamp
                  v-if="fixBatch.startedAt"
                  :date="new Date(fixBatch.startedAt)"
                  size="long"
                />
                <div v-else>No timestamp available.</div>
              </div>
            </div>
          </div>
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
        <div class="overflow-auto">
          <div
            v-for="fix in selectedFixBatchInfo.fixes"
            :key="fix.attributeValueId"
            :class="
              fix.actionKind === selectedFix?.actionKind
                ? 'bg-action-500'
                : 'hover:bg-black'
            "
            class="p-xs h-12 cursor-pointer flex flex-row items-center"
            @click="selectFix(fix.actionKind)"
          >
            <StatusIndicatorIcon
              type="resource"
              :status="fix.resource?.status ?? 'unknown'"
            />
            <div class="font-bold pl-xs line-clamp-2">
              {{ `${formatTitle(fix.actionKind)} ${fix.schemaName}` }}
            </div>
          </div>
        </div>
      </div>
      <div
        v-if="selectedFixInfo && selectedFixInfo.resource"
        class="bg-shade-100 grow p-4"
      >
        <CodeViewer
          :code="
            selectedFixInfo.resource.data
              ? JSON.stringify(selectedFixInfo.resource.data, null, 2)
              : ''
          "
          class="text-neutral-50"
          titleClasses=""
        >
          <template #title>
            <StatusIndicatorIcon
              type="resource"
              :status="selectedFixInfo.resource.status"
            />
            <div class="grow flex flex-col pl-xs">
              <div class="font-bold">
                {{
                  `${formatTitle(selectedFixInfo.actionKind)} ${
                    selectedFixInfo.schemaName
                  }`
                }}
              </div>
              <div>
                {{
                  selectedFixInfo.resource.message
                    ? selectedFixInfo.resource.message
                    : `Health ${selectedFixInfo.resource.status}`
                }}
              </div>
            </div>

            <div class="pr-xs">
              <FixDetails
                v-if="
                  selectedFixInfo.resource.logs &&
                  selectedFixInfo.resource.logs.length > 0
                "
                :health="selectedFixInfo.resource.status"
                :message="
                  [
                    `${formatTitle(selectedFixInfo.actionKind)} ${
                      selectedFixInfo.schemaName
                    }`,
                    selectedFixInfo.resource.message ?? '',
                  ].filter((f) => f.length > 0)
                "
                :details="selectedFixInfo.resource.logs"
              />
            </div>
          </template>
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
      <p class="w-full text-3xl text-neutral-500">
        {{
          fixListDisplay.length > 0 ? "No Fix Run Selected" : "No Fixes Run Yet"
        }}
      </p>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { ref, computed } from "vue";
import { Timestamp } from "@si/vue-lib/design-system";
import { useFixesStore } from "@/store/fixes.store";
import CodeViewer from "@/components/CodeViewer.vue";
import StatusIndicatorIcon from "@/components/StatusIndicatorIcon.vue";
import FixDetails from "@/components/FixDetails.vue";

export interface SortOption {
  value: string;
  title: string;
}
interface SelectedFix {
  actionKind: string;
}

const sortOptions: SortOption[] = [
  { value: "r", title: "Newest" },
  { value: "o", title: "Oldest" },
];
const selectedSort = ref<SortOption | undefined>(sortOptions[0]);
const selectedFixBatchId = ref<string | null>(null);
const selectedFix = ref<SelectedFix | null>(null);
const selectFixBatch = (id: string) => {
  selectedFixBatchId.value = id;
  selectedFix.value = null;
};
const selectFix = (actionKind: string) => {
  selectedFix.value = {
    actionKind,
  };
};
const fixesStore = useFixesStore();
const fixBatchesWithFixes = computed(() =>
  fixesStore.allFinishedFixBatches.map((batch) => ({
    ...batch,
    fixes: fixesStore.fixesOnBatch(batch.id),
  })),
);
const fixListDisplay = computed(() => {
  if (selectedSort.value?.value === "r") {
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
      (fix) => fix.actionKind === selectedFix.value?.actionKind,
    );
  }
  return null;
});

const formatTitle = (title: string) => {
  return title
    .split(" ")
    .map((t) => `${t[0]?.toUpperCase()}${t.slice(1).toLowerCase()}`)
    .join(" ");
};
</script>
