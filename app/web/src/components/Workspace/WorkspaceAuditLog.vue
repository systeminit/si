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
              :filters="logsStore.filters"
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
              :expanded="rowCollapseState[Number(row.id)]"
            />
            <tr class="invisible"></tr>
          </template>
        </tbody>
      </table>
      <template v-if="initialLoadLogsRequestStatus.isSuccess">
        <span
          v-if="noRowsMessage"
          class="flex flex-row items-center justify-center pt-md"
        >
          {{ noRowsMessage }}
        </span>
        <div class="flex flex-row items-center justify-center py-md">
          <VButton
            size="xs"
            tone="action"
            class="grow max-w-md flex-row"
            :disabled="!canLoadMore"
            :label="canLoadMore ? 'Load 50 More' : 'All Entries Loaded'"
            loadingText="Loading More Logs..."
            :requestStatus="loadLogsRequestStatus"
            @click="loadLogs(true, false)"
          />
        </div>
      </template>
      <RequestStatusMessage
        v-else
        :requestStatus="initialLoadLogsRequestStatus"
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
  getFilteredRowModel,
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
const sizeForWatcher = computed(() => logsStore.size);
const canLoadMore = computed(() => logsStore.canLoadMore);

const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const rowCollapseState = ref(new Array(logs.value.length).fill(false));
const anyRowsOpen = computed(() => rowCollapseState.value.some(Boolean));
const toggleRowExpand = (id: number) => {
  rowCollapseState.value[id] = !rowCollapseState.value[id];
};
const collapseAllRows = () => {
  rowCollapseState.value = new Array(logs.value.length).fill(false);
};

const initialLoadLogsRequestIdentifier = "initialLoadLogs";
const initialLoadLogsRequestStatus = logsStore.getRequestStatus(
  "LOAD_PAGE",
  initialLoadLogsRequestIdentifier,
);
const performInitialLoadLogs = async () => {
  collapseAllRows();
  const size = logsStore.size;
  const sortAscending = logsStore.sortAscending;
  logsStore.LOAD_PAGE(size, sortAscending, initialLoadLogsRequestIdentifier);
  trackEvent("load-audit-logs", { size, sortAscending });
};

const loadLogsRequestIdentifier = "loadLogs";
const loadLogsRequestStatus = logsStore.getRequestStatus(
  "LOAD_PAGE",
  loadLogsRequestIdentifier,
);
const loadLogs = async (expandSize: boolean, toggleTimestampSort: boolean) => {
  if (expandSize === true) {
    logsStore.size += 50;
  }
  if (toggleTimestampSort) {
    logsStore.sortAscending = !logsStore.sortAscending;
  }

  const size = logsStore.size;
  const sortAscending = logsStore.sortAscending;

  logsStore.LOAD_PAGE(size, sortAscending, loadLogsRequestIdentifier);
  trackEvent("load-audit-logs", { size, sortAscending });
};

// Load the logs when this component is loaded.
performInitialLoadLogs();

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
    filterFn: "arrIncludesSome",
  }),
  columnHelper.accessor("entityType", {
    header: "Entity Type",
    cell: (info) => info.getValue(),
    filterFn: "arrIncludesSome",
  }),
  columnHelper.accessor("entityName", {
    header: "Entity Name",
    cell: (info) => info.getValue(),
    filterFn: "arrIncludesSome",
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
    filterFn: "arrIncludesSome",
  }),
  columnHelper.accessor("userName", {
    header: "User",
    cell: (info) => info.getValue(),
    filterFn: "arrIncludesSome",
  }),
  columnHelper.accessor("timestamp", {
    header: "Time",
    cell: (info) =>
      h(Timestamp, {
        date: info.getValue(),
        relative: "standard",
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
    return logs.value;
  },
  initialState: {
    columnVisibility: {
      changeSetId: false,
    },
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
});

table.setPageSize(sizeForWatcher.value);
watch(sizeForWatcher, (sizeForWatcher) => {
  table.setPageSize(sizeForWatcher);
});

const onHeaderClick = (id: string) => {
  if (id === "timestamp") {
    loadLogs(false, true);
  } else if (id === "json" && anyRowsOpen.value) {
    collapseAllRows();
  }
};

const toggleFilter = (id: string, filterId: string) => {
  if (id === "changeSetName") {
    if (logsStore.filters.changeSetFilter.includes(filterId)) {
      const i = logsStore.filters.changeSetFilter.indexOf(filterId);
      logsStore.filters.changeSetFilter.splice(i, 1);
    } else logsStore.filters.changeSetFilter.push(filterId);
    table.getColumn(id)?.setFilterValue(logsStore.filters.changeSetFilter);
  } else if (id === "entityName") {
    if (logsStore.filters.entityNameFilter.includes(filterId)) {
      const i = logsStore.filters.entityNameFilter.indexOf(filterId);
      logsStore.filters.entityNameFilter.splice(i, 1);
    } else logsStore.filters.entityNameFilter.push(filterId);
    table.getColumn(id)?.setFilterValue(logsStore.filters.entityNameFilter);
  } else if (id === "entityType") {
    if (logsStore.filters.entityTypeFilter.includes(filterId)) {
      const i = logsStore.filters.entityTypeFilter.indexOf(filterId);
      logsStore.filters.entityTypeFilter.splice(i, 1);
    } else logsStore.filters.entityTypeFilter.push(filterId);
    table.getColumn(id)?.setFilterValue(logsStore.filters.entityTypeFilter);
  } else if (id === "title") {
    if (logsStore.filters.titleFilter.includes(filterId)) {
      const i = logsStore.filters.titleFilter.indexOf(filterId);
      logsStore.filters.titleFilter.splice(i, 1);
    } else logsStore.filters.titleFilter.push(filterId);
    table.getColumn(id)?.setFilterValue(logsStore.filters.titleFilter);
  } else if (id === "userName") {
    if (logsStore.filters.userFilter.includes(filterId)) {
      const i = logsStore.filters.userFilter.indexOf(filterId);
      logsStore.filters.userFilter.splice(i, 1);
    } else logsStore.filters.userFilter.push(filterId);
    table.getColumn(id)?.setFilterValue(logsStore.filters.userFilter);
  }
};

const clearFilters = (id: string) => {
  if (id === "changeSetName") {
    logsStore.filters.changeSetFilter = [];
  } else if (id === "entityName") {
    logsStore.filters.entityNameFilter = [];
  } else if (id === "entityType") {
    logsStore.filters.entityTypeFilter = [];
  } else if (id === "title") {
    logsStore.filters.titleFilter = [];
  } else if (id === "userName") {
    logsStore.filters.userFilter = [];
  }
  table.setColumnFilters((filters) =>
    filters.filter((filter) => filter.id !== id),
  );
};

const noRowsMessage = computed(() => {
  if (logs.value.length < 1)
    return "No logs exist for the selected Change Set.";
  if (table.getRowModel().rows.length === 0)
    return "No entries match selected filter criteria.";
  return null;
});

// NOTE(nick): restore pagination after audit trail is shipped.
// const canGetPreviousPage = () => {
//   return logsStore.filters.page > 1;
// };
//
// const getCanNextPage = () => {
//   return logsStore.filters.page < totalPages.value;
// };
//
// const setPage = (pageNumber: number) => {
//   logsStore.filters.page = pageNumber;
//   loadLogs();
// };
//
// const nextPage = () => {
//   logsStore.filters.page++;
//   loadLogs();
// };
//
// const previousPage = () => {
//   logsStore.filters.page--;
//   loadLogs();
// };
//
// const currentPage = computed(() =>
//   totalPages.value === 0 ? 0 : logsStore.filters.page,
// );
</script>
