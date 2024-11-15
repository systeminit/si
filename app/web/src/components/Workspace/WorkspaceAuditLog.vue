<template>
  <div
    class="w-full h-full min-h-0 flex flex-col overflow-hidden items-center relative dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <ScrollArea>
      <template #top>
        <div :class="clsx('w-full flex-none')">
          <div class="flex items-center gap-2xs p-xs">
            <Icon name="eye" class="flex-none" />
            <div
              v-if="changeSetsStore.headSelected"
              class="flex-grow text-lg font-bold truncate"
            >
              Audit Logs for HEAD
            </div>
            <div
              v-else-if="selectedChangeSetName"
              class="flex-grow text-lg font-bold truncate"
            >
              Audit Logs for Change Set: {{ selectedChangeSetName }}
            </div>
            <div v-else class="flex-grow text-lg font-bold truncate">
              Audit Logs for Selected Change Set
            </div>

            <!-- <div -->
            <!-- class="flex items-center gap-2xs pr-xs whitespace-nowrap flex-none" -->
            <!-- > -->
            <!-- <div>Page</div> -->
            <!-- <div class="font-bold">{{ currentPage }} of {{ totalPages }}</div> -->
            <!-- </div> -->

            <!-- NOTE(nick): restore pagination once the audit trail is shipped.
            <IconButton
              v-tooltip="
                !canGetPreviousPage() ? 'You are on the first page.' : undefined
              "
              icon="double-arrow-left"
              iconTone="shade"
              :disabled="!canGetPreviousPage()"
              @click="() => setPage(1)"
            />
            <IconButton
              v-tooltip="
                !canGetPreviousPage() ? 'You are on the first page.' : undefined
              "
              icon="chevron--left"
              iconTone="shade"
              :disabled="!canGetPreviousPage()"
              @click="() => previousPage()"
            />
            <IconButton
              v-tooltip="
                !getCanNextPage() ? 'You are on the last page.' : undefined
              "
              icon="chevron--right"
              iconTone="shade"
              :disabled="!getCanNextPage()"
              @click="() => nextPage()"
            />
            <IconButton
              v-tooltip="
                !getCanNextPage() ? 'You are on the last page.' : undefined
              "
              icon="double-arrow-left"
              rotate="down"
              iconTone="shade"
              :disabled="!getCanNextPage()"
              @click="() => setPage(totalPages)"
            />
            -->

            <!-- <span class="flex items-center gap-1">
              | Go to page:
              <input
                type="number"
                :value="goToPageNumber"
                class="border p-1 rounded w-16"
                @change="handleGoToPage"
              />
            </span> -->
          </div>
          <!-- <div>{{ table.getRowModel().rows.length }} Rows</div>
        <pre>{{ JSON.stringify(table.getState().pagination, null, 2) }}</pre> -->
          <!-- <div class="h-2" />
      <button class="border p-2" @click="rerender">Rerender</button> -->
        </div>
      </template>
      <table class="w-full relative border-collapse">
        <thead>
          <tr
            v-for="headerGroup in table.getHeaderGroups()"
            :key="headerGroup.id"
          >
            <AuditLogHeader
              v-for="header in headerGroup.headers"
              :key="header.id"
              :header="header"
              :filters="currentFilters"
              :anyRowsOpen="anyRowsOpen"
              @select="onHeaderClick(header.id)"
              @clearFilters="clearFilters(header.id)"
              @toggleFilter="(f) => toggleFilter(header.id, f)"
            />
          </tr>
        </thead>
        <tbody>
          <template v-for="row in table.getRowModel().rows" :key="row.id">
            <tr
              :class="
                clsx(
                  'h-lg text-sm hover:border',
                  themeClasses(
                    'hover:border-action-500',
                    'hover:border-action-300',
                  ),
                  rowCollapseState[Number(row.id)]
                    ? themeClasses('bg-action-200', 'bg-action-900')
                    : themeClasses(
                        'odd:bg-neutral-200 even:bg-neutral-100',
                        'odd:bg-neutral-700 even:bg-neutral-800',
                      ),
                )
              "
            >
              <AuditLogCell
                v-for="cell in row.getVisibleCells()"
                :key="cell.id"
                :cell="cell"
                :rowExpanded="rowCollapseState[Number(cell.row.id)]"
                @toggleExpand="toggleRowExpand(Number(cell.row.id))"
              />
            </tr>
            <AuditLogDrawer
              :row="row"
              :colspan="columns.length"
              :json="JSON.stringify(logs[Number(row.id)], null, 2)"
              :expanded="rowCollapseState[Number(row.id)]"
            />
            <tr class="invisible"></tr>
          </template>
        </tbody>
      </table>
      <template v-if="initialLoadRequestStatus.isSuccess">
        <span
          v-if="filteredLogs.length < 1"
          class="flex flex-row items-center justify-center pt-md"
        >
          No entries match selected filter criteria.
        </span>
        <div class="flex flex-row items-center justify-center py-md">
          <VButton
            size="xs"
            tone="action"
            class="grow max-w-md flex-row"
            :disabled="!canLoadMore"
            :label="canLoadMore ? 'Load 50 More' : 'All Entries Loaded'"
            loadingText="Loading More Logs..."
            :requestStatus="loadMoreRequestStatus"
            @click="loadMore()"
          />
        </div>
      </template>
      <RequestStatusMessage
        v-else
        :requestStatus="initialLoadRequestStatus"
        loadingMessage="Loading Logs..."
      />
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import {
  Icon,
  RequestStatusMessage,
  ScrollArea,
  themeClasses,
  Timestamp,
  VButton,
} from "@si/vue-lib/design-system";
import {
  getCoreRowModel,
  getPaginationRowModel,
  useVueTable,
  createColumnHelper,
} from "@tanstack/vue-table";
import clsx from "clsx";
import { h, computed, ref, withDirectives, resolveDirective, watch } from "vue";
import { trackEvent } from "@/utils/tracking";
import { AuditLogDisplay, useLogsStore } from "@/store/logs.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import AuditLogHeader from "../AuditLogHeader.vue";
import AuditLogCell from "../AuditLogCell.vue";
import AuditLogDrawer from "../AuditLogDrawer.vue";

const changeSetsStore = useChangeSetsStore();
const logsStore = useLogsStore();

const logs = computed(() => logsStore.logs);
const size = computed(() => logsStore.size);
const canLoadMore = computed(() => logsStore.canLoadMore);
const currentFilters = computed(() => logsStore.filters);

const filteredLogs = computed(() => {
  const result = [];
  for (const log of logs.value) {
    if (currentFilters.value.changeSetFilter.length > 0) {
      if (!log.changeSetName) {
        continue;
      } else if (
        !currentFilters.value.changeSetFilter.includes(log.changeSetName)
      ) {
        continue;
      }
    }
    if (
      currentFilters.value.entityNameFilter.length > 0 &&
      !currentFilters.value.entityNameFilter.includes(log.entityName)
    ) {
      continue;
    }
    if (
      currentFilters.value.entityTypeFilter.length > 0 &&
      !currentFilters.value.entityTypeFilter.includes(log.entityType)
    ) {
      continue;
    }
    if (
      currentFilters.value.titleFilter.length > 0 &&
      !currentFilters.value.titleFilter.includes(log.title)
    ) {
      continue;
    }
    if (
      currentFilters.value.userFilter.length > 0 &&
      !currentFilters.value.userFilter.includes(log.userName)
    ) {
      continue;
    }
    result.push(log);
  }
  return result;
});

const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const rowCollapseState = ref(new Array(filteredLogs.value.length).fill(false));
const anyRowsOpen = computed(() => rowCollapseState.value.some(Boolean));
const toggleRowExpand = (id: number) => {
  rowCollapseState.value[id] = !rowCollapseState.value[id];
};
const collapseAllRows = () => {
  rowCollapseState.value = new Array(filteredLogs.value.length).fill(false);
};

// TODO(nick): restore pagination once the audit trail feature is shipped.
// const loadLogs = async () => {
//   collapseAllRows();
//   logsStore.LOAD_PAGE(size.value);
//   trackEvent("load-audit-logs", { size: size.value });
// };

const initialLoadRequestIdentifier = "initialLoad";
const initialLoadRequestStatus = logsStore.getRequestStatus(
  "LOAD_PAGE",
  initialLoadRequestIdentifier,
);
const performInitialLoad = async () => {
  collapseAllRows();
  logsStore.LOAD_PAGE(size.value, initialLoadRequestIdentifier);
  trackEvent("load-audit-logs", { size: size.value });
};

const loadMoreRequestIdentifier = "loadMore";
const loadMoreRequestStatus = logsStore.getRequestStatus(
  "LOAD_PAGE",
  loadMoreRequestIdentifier,
);
const loadMore = async () => {
  logsStore.size += 50;
  const newSize = logsStore.size;
  logsStore.LOAD_PAGE(newSize, loadMoreRequestIdentifier);
  trackEvent("load-audit-logs", { size: newSize });
};

// Load the logs when this component is loaded.
performInitialLoad();

const columnHelper = createColumnHelper<AuditLogDisplay>();

// NOTE(nick): restore pagination after audit trail is shipped.
// const totalPages = computed(() => Math.ceil(logsStore.total / PAGE_SIZE));

const columns = [
  {
    id: "json",
    header: "",
    cell: "",
  },
  columnHelper.accessor("title", {
    header: "Event",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("entityType", {
    header: "Entity Type",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("entityName", {
    header: "Entity Name",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("changeSetName", {
    header: "Change Set",
    cell: (info) =>
      withDirectives(
        h("div", {
          innerText: info.getValue(),
          class: "hover:underline cursor-pointer",
        }),
        [[resolveDirective("tooltip"), info.row.getValue("changeSetId")]],
      ),
  }),
  columnHelper.accessor("userName", {
    header: "User",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("timestamp", {
    header: "Time",
    cell: (info) =>
      h(Timestamp, {
        date: info.getValue(),
        relative: true,
        enableDetailTooltip: true,
        refresh: true,
      }),
  }),
  columnHelper.accessor("changeSetId", {
    header: "Change Set Id",
    cell: (info) => info.getValue(),
  }),
];

const table = useVueTable({
  get data() {
    return filteredLogs.value;
  },
  initialState: {
    columnVisibility: {
      changeSetId: false,
    },
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
});

table.setPageSize(size.value);
watch(size, (size) => {
  table.setPageSize(size);
});

const onHeaderClick = (id: string) => {
  if (id === "timestamp") {
    // NOTE(nick): restore timestamp sort after the audit trail feature is shipped.
    // currentFilters.value.sortTimestampAscending = !currentFilters.value.sortTimestampAscending;
    // loadLogs();
  } else if (id === "json" && anyRowsOpen.value) {
    collapseAllRows();
  }
};

const toggleFilter = (id: string, filterId: string) => {
  if (id === "changeSetName") {
    if (currentFilters.value.changeSetFilter.includes(filterId)) {
      const i = currentFilters.value.changeSetFilter.indexOf(filterId);
      currentFilters.value.changeSetFilter.splice(i, 1);
    } else currentFilters.value.changeSetFilter.push(filterId);
  } else if (id === "entityName") {
    if (currentFilters.value.entityNameFilter.includes(filterId)) {
      const i = currentFilters.value.entityNameFilter.indexOf(filterId);
      currentFilters.value.entityNameFilter.splice(i, 1);
    } else currentFilters.value.entityNameFilter.push(filterId);
  } else if (id === "entityType") {
    if (currentFilters.value.entityTypeFilter.includes(filterId)) {
      const i = currentFilters.value.entityTypeFilter.indexOf(filterId);
      currentFilters.value.entityTypeFilter.splice(i, 1);
    } else currentFilters.value.entityTypeFilter.push(filterId);
  } else if (id === "title") {
    if (currentFilters.value.titleFilter.includes(filterId)) {
      const i = currentFilters.value.titleFilter.indexOf(filterId);
      currentFilters.value.titleFilter.splice(i, 1);
    } else currentFilters.value.titleFilter.push(filterId);
  } else if (id === "userName") {
    if (currentFilters.value.userFilter.includes(filterId)) {
      const i = currentFilters.value.userFilter.indexOf(filterId);
      currentFilters.value.userFilter.splice(i, 1);
    } else currentFilters.value.userFilter.push(filterId);
  }
};

const clearFilters = (id: string) => {
  if (id === "changeSetName") {
    currentFilters.value.changeSetFilter = [];
  } else if (id === "entityName") {
    currentFilters.value.entityNameFilter = [];
  } else if (id === "entityType") {
    currentFilters.value.entityTypeFilter = [];
  } else if (id === "title") {
    currentFilters.value.titleFilter = [];
  } else if (id === "userName") {
    currentFilters.value.userFilter = [];
  }
};

// NOTE(nick): restore pagination after audit trail is shipped.
// const canGetPreviousPage = () => {
//   return currentFilters.value.page > 1;
// };
//
// const getCanNextPage = () => {
//   return currentFilters.value.page < totalPages.value;
// };
//
// const setPage = (pageNumber: number) => {
//   currentFilters.value.page = pageNumber;
//   loadLogs();
// };
//
// const nextPage = () => {
//   currentFilters.value.page++;
//   loadLogs();
// };
//
// const previousPage = () => {
//   currentFilters.value.page--;
//   loadLogs();
// };
//
// const currentPage = computed(() =>
//   totalPages.value === 0 ? 0 : currentFilters.value.page,
// );
</script>
