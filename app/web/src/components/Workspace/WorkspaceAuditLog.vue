<template>
  <div
    class="w-full h-full min-h-0 flex flex-col overflow-hidden items-center relative dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <ScrollArea>
      <template #top>
        <div :class="clsx('w-full flex-none')">
          <div class="flex items-center gap-xs p-xs">
            <Icon name="eye" class="flex-none" />
            <div class="flex-grow text-lg font-bold">Audit Logs</div>
            <div class="flex items-center gap-2xs">
              <div>Page</div>
              <div class="font-bold">
                {{ table.getState().pagination.pageIndex + 1 }} of
                {{ table.getPageCount() }}
              </div>
            </div>
            <button
              class="border rounded p-1"
              :disabled="!table.getCanPreviousPage()"
              @click="() => table.setPageIndex(0)"
            >
              «
            </button>
            <button
              class="border rounded p-1"
              :disabled="!table.getCanPreviousPage()"
              @click="() => table.previousPage()"
            >
              ‹
            </button>
            <button
              class="border rounded p-1"
              :disabled="!table.getCanNextPage()"
              @click="() => table.nextPage()"
            >
              ›
            </button>
            <button
              class="border rounded p-1"
              :disabled="!table.getCanNextPage()"
              @click="() => table.setPageIndex(table.getPageCount() - 1)"
            >
              »
            </button>
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
      <table class="w-full relative">
        <thead>
          <tr
            v-for="headerGroup in table.getHeaderGroups()"
            :key="headerGroup.id"
          >
            <th
              v-for="header in headerGroup.headers"
              :key="header.id"
              :colSpan="header.colSpan"
              :class="
                clsx(
                  'h-8 sticky top-0',
                  themeClasses('bg-shade-0', 'bg-shade-100'),
                )
              "
            >
              <FlexRender
                v-if="!header.isPlaceholder"
                :render="header.column.columnDef.header"
                :props="header.getContext()"
              />
            </th>
          </tr>
        </thead>
        <tbody v-if="logLoadingRequestStatus.isSuccess">
          <tr
            v-for="row in table.getRowModel().rows"
            :key="row.id"
            :class="
              clsx(
                'h-md text-sm',
                themeClasses(
                  'odd:bg-neutral-200 even:bg-neutral-100',
                  'odd:bg-neutral-700 even:bg-neutral-800',
                ),
              )
            "
          >
            <td
              v-for="cell in row.getVisibleCells()"
              :key="cell.id"
              align="center"
            >
              <FlexRender
                :render="cell.column.columnDef.cell"
                :props="cell.getContext()"
              />
            </td>
          </tr>
        </tbody>
      </table>
      <RequestStatusMessage
        v-if="!logLoadingRequestStatus.isSuccess"
        :requestStatus="logLoadingRequestStatus"
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
} from "@si/vue-lib/design-system";
import {
  FlexRender,
  getCoreRowModel,
  getPaginationRowModel,
  useVueTable,
  createColumnHelper,
} from "@tanstack/vue-table";
import clsx from "clsx";
import { h, ref, computed } from "vue";
import { AuditLogDisplay, LogFilters, useLogsStore } from "@/store/logs.store";

// const range = (len: number) => {
//   const arr: number[] = [];
//   for (let i = 0; i < len; i++) {
//     arr.push(i);
//   }
//   return arr;
// };

// const dummyRow = (): AuditLogDisplay => {
//   return {
//     actorId: "system",
//     actorName: "system",
//     service: "sdf",
//     kind: "testkind",
//     timestamp: "2024-10-15T22:06:42+0000",
//     ip: "127.0.0.1",
//     changeSetId: "testchangesetid",
//     changeSetName: "testchangesetname",
//   };
// };

// function makeData(...lens: number[]) {
//   const makeDataLevel = (depth = 0): AuditLogDisplay[] => {
//     // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
//     const len = lens[depth]!;
//     return range(len).map((): AuditLogDisplay => {
//       return dummyRow();
//     });
//   };

//   return makeDataLevel();
// }

// const INITIAL_PAGE_INDEX = 0;
// const goToPageNumber = ref(INITIAL_PAGE_INDEX + 1);

const PAGE_SIZE = 50;
const DEFAULT_FILTERS = {
  page: 1,
  pageSize: PAGE_SIZE,
  sortTimestampAscending: true,
  excludeSystemUser: false,
  kindFilter: [],
  serviceFilter: [],
  changeSetFilter: [],
  userFilter: [],
} as LogFilters;

const logsStore = useLogsStore();
logsStore.LOAD_PAGE(DEFAULT_FILTERS);
const logLoadingRequestStatus = logsStore.getRequestStatus("LOAD_PAGE");

const columnHelper = createColumnHelper<AuditLogDisplay>();
const logs = computed(() => logsStore.logs);

const columns = [
  columnHelper.accessor("timestamp", {
    header: "Timestamp",
    cell: (info) => h(Timestamp, { date: info.getValue(), relative: true }),
  }),
  columnHelper.accessor("changeSetName", {
    header: "Change Set",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("kind", {
    header: "Event Kind",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("actorName", {
    header: "Actor",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("ip", {
    header: "Origin IP Address",
    cell: (info) => info.getValue(),
  }),
  columnHelper.accessor("service", {
    header: "Service",
    cell: (info) => info.getValue(),
  }),
];

const table = useVueTable({
  get data() {
    return logs.value;
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
});
table.setPageSize(PAGE_SIZE);

// eslint-disable-next-line @typescript-eslint/no-explicit-any
// function handleGoToPage(e: any) {
//   const page = e.target.value ? Number(e.target.value) - 1 : 0;
//   goToPageNumber.value = page + 1;
//   table.setPageIndex(page);
// }
</script>
