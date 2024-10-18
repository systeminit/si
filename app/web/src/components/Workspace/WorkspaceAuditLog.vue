<template>
  <div
    class="w-full h-full min-h-0 flex flex-col overflow-hidden items-center relative dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <ScrollArea>
      <template #top>
        <div :class="clsx('w-full flex-none')">
          <div class="flex items-center gap-2xs p-xs">
            <Icon name="eye" class="flex-none" />
            <div class="flex-grow text-lg font-bold">
              Audit Logs (this feature is not complete, mock data)
            </div>
            <div class="flex items-center gap-2xs pr-xs">
              <div>Page</div>
              <div class="font-bold">
                {{ currentFilters.page }} of {{ totalPages }}
              </div>
            </div>
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
      <table v-if="logLoadingRequestStatus.isSuccess" class="w-full relative">
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
              :users="users"
              @select="onHeaderClick(header.id)"
              @clearFilters="clearFilters(header.id)"
              @toggleFilter="(f) => toggleFilter(header.id, f)"
            />
          </tr>
        </thead>
        <tbody>
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
              :class="
                clsx(
                  'border-x border-collapse',
                  themeClasses('border-neutral-300', 'border-neutral-900'),
                )
              "
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
        v-else
        :requestStatus="logLoadingRequestStatus"
        loadingMessage="Loading Logs..."
      />
    </ScrollArea>
  </div>
</template>

<script lang="ts" setup>
import {
  Icon,
  IconButton,
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
import { h, computed, ref } from "vue";
import { AuditLogDisplay, LogFilters, useLogsStore } from "@/store/logs.store";
import { AdminUser, useAdminStore } from "@/store/admin.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import AuditLogHeader from "../AuditLogHeader.vue";

const adminStore = useAdminStore();
const workspacesStore = useWorkspacesStore();
const changeSetsStore = useChangeSetsStore();

const users = ref([] as AdminUser[]);

const PAGE_SIZE = 50; // Currently this is fixed, might make it variable later
const DEFAULT_FILTERS = {
  page: 1,
  pageSize: PAGE_SIZE,
  sortTimestampAscending: false,
  excludeSystemUser: false,
  kindFilter: [],
  serviceFilter: [],
  changeSetFilter: [changeSetsStore.selectedChangeSetId],
  userFilter: [],
} as LogFilters;
const currentFilters = ref({ ...DEFAULT_FILTERS });
const logsStore = useLogsStore();
const loadLogs = async () => {
  logsStore.LOAD_PAGE(currentFilters.value);
  if (workspacesStore.urlSelectedWorkspaceId) {
    const result = await adminStore.LIST_WORKSPACE_USERS(
      workspacesStore.urlSelectedWorkspaceId,
    );
    if (result?.result.success) {
      users.value = result.result.data.users;
      return;
    }
  }
  users.value = [];
};
loadLogs();
const logLoadingRequestStatus = logsStore.getRequestStatus("LOAD_PAGE");

const columnHelper = createColumnHelper<AuditLogDisplay>();
const logs = computed(() => logsStore.logs);
const totalPages = computed(() => Math.ceil(logsStore.total / PAGE_SIZE));

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
  // columnHelper.accessor("service", {
  //   header: "Service",
  //   cell: (info) => info.getValue(),
  // }),
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

const onHeaderClick = (id: string) => {
  if (id === "timestamp") {
    currentFilters.value.sortTimestampAscending =
      !currentFilters.value.sortTimestampAscending;
    loadLogs();
  }
};

const toggleFilter = (id: string, filterId: string) => {
  if (id === "kind") {
    if (currentFilters.value.kindFilter.includes(filterId)) {
      const i = currentFilters.value.kindFilter.indexOf(filterId);
      currentFilters.value.kindFilter.splice(i, 1);
    } else currentFilters.value.kindFilter.push(filterId);
  } else if (id === "service") {
    if (currentFilters.value.serviceFilter.includes(filterId)) {
      const i = currentFilters.value.serviceFilter.indexOf(filterId);
      currentFilters.value.serviceFilter.splice(i, 1);
    } else currentFilters.value.serviceFilter.push(filterId);
  } else if (id === "changeSetName") {
    if (currentFilters.value.changeSetFilter.includes(filterId)) {
      const i = currentFilters.value.changeSetFilter.indexOf(filterId);
      currentFilters.value.changeSetFilter.splice(i, 1);
    } else currentFilters.value.changeSetFilter.push(filterId);
  } else if (id === "actorName") {
    if (currentFilters.value.userFilter.includes(filterId)) {
      const i = currentFilters.value.userFilter.indexOf(filterId);
      currentFilters.value.userFilter.splice(i, 1);
    } else currentFilters.value.userFilter.push(filterId);
  }
  loadLogs();
};

const clearFilters = (id: string) => {
  if (id === "kind") {
    currentFilters.value.kindFilter = [];
  } else if (id === "service") {
    currentFilters.value.serviceFilter = [];
  } else if (id === "changeSetName") {
    currentFilters.value.changeSetFilter = [];
  } else if (id === "actorName") {
    currentFilters.value.userFilter = [];
  }
  loadLogs();
};

const canGetPreviousPage = () => {
  return currentFilters.value.page > 1;
};

const getCanNextPage = () => {
  return currentFilters.value.page < totalPages.value;
};

const setPage = (pageNumber: number) => {
  currentFilters.value.page = pageNumber;
  loadLogs();
};

const nextPage = () => {
  currentFilters.value.page++;
  loadLogs();
};

const previousPage = () => {
  currentFilters.value.page--;
  loadLogs();
};
</script>
