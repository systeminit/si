<template>
  <Stack
    :class="
      clsx(
        '[&_dd]:m-xs [&_dd]:pl-xs [&_dd]:break-all [&_dd]:font-mono',
        '[&_dt]:p-2xs [&_dt]:font-bold',
        themeClasses(
          '[&_select]:text-neutral-900 [&_select]:bg-neutral-100',
          '[&_select]:text-neutral-200 [&_select]:bg-neutral-900',
        ),
      )
    "
  >
    <h2 class="font-bold text-xl">WORKSPACES</h2>
    <div class="flex flex-row gap-xs p-xs w-full">
      <LoadStatus
        :requestStatus="searchWorkspacesReqStatus"
        loadingMessage="Searching workspaces ..."
      >
        <template #success>
          <select
            v-if="filteredWorkspaces?.length > 0"
            v-model="selectedWorkspaceId"
          >
            <option
              v-for="workspace in filteredWorkspaces"
              :key="workspace.id"
              :value="workspace.id"
            >
              {{ workspace.name }} ({{ workspace.id }})
            </option>
          </select>
          <p v-else>No workspaces found...</p>
        </template>
      </LoadStatus>
      <VormInput
        v-model="workspacesFilter"
        label="WORKSPACE SEARCH: type here to search for a workspace by id or name, or change set id or name, or user id, name, or email (50 results max). then press ENTER"
        placeholder="workspace name/id, or user name/id/email)"
        @keydown.enter="searchWorkspaces(workspacesFilter)"
      />
    </div>
    <template v-if="selectedWorkspaceId && selectedWorkspace">
      <LoadStatus
        :requestStatus="listChangeSetsReqStatus.value"
        loadingMessage="Loading change sets ..."
      >
        <template #success>
          <div class="flex flex-row gap-xs p-xs w-full">
            <select
              v-if="filteredChangeSets?.length > 0"
              v-model="selectedChangeSetId"
            >
              <option
                v-for="changeSet in filteredChangeSets"
                :key="changeSet.id"
                :value="changeSet.id"
              >
                {{ changeSet.name }} ({{ changeSet.id }}) --
                {{ changeSet.status }}
                <p
                  v-if="changeSet.id === selectedWorkspace?.defaultChangeSetId"
                >
                  *
                </p>
              </option>
            </select>
            <p v-else>No change sets found for workspace ...</p>
            <VormInput
              v-model="changeSetsFilter"
              placeholder="change set name, ID, status, e.g. 'open'"
              label="CHANGE SET SEARCH: type here to filter the change set list (e.g., type 'open' to see only open change sets)"
            />
          </div>
        </template>
      </LoadStatus>

      <div class="flex flex-row flex-wrap gap-xs p-xs w-full">
        <Stack
          v-if="selectedChangeSetId && selectedChangeSet"
          class="flex-none"
        >
          <VButton
            :loading="isGettingSnapshot"
            @click="getSnapshot(selectedWorkspaceId, selectedChangeSetId)"
            >Save snapshot to disk</VButton
          >

          <VButton
            :loading="isSettingSnapshot"
            @click="setSnapshot(selectedWorkspaceId, selectedChangeSetId)"
            >Replace snapshot for this change set</VButton
          >

          <VButton
            :loading="isGettingCasData"
            @click="getCasData(selectedWorkspaceId, selectedChangeSetId)"
            >Save cas data to disk</VButton
          >

          <VButton
            :loading="isUploadingCasData"
            @click="uploadCasData(selectedWorkspaceId, selectedChangeSetId)"
            >Upload cas data to service</VButton
          >

          <VButton
            :requestStatus="validateSnapshotRequestStatus"
            @click="validateSnapshot(selectedWorkspaceId, selectedChangeSetId)"
            >Validate snapshot</VButton
          >

          <VButton
            :requestStatus="migrateConnectionsRequestStatus"
            @click="
              migrateConnections(selectedWorkspaceId, selectedChangeSetId)
            "
            >Migrate connections</VButton
          >

          <VButton
            :requestStatus="migrateConnectionsRequestStatus"
            @click="
              migrateConnections(selectedWorkspaceId, selectedChangeSetId, {
                dryRun: true,
              })
            "
            >Migrate connections (dry run only)</VButton
          >
        </Stack>

        <dl class="p-3">
          <dt>Workspace Id</dt>
          <dd>
            {{ selectedWorkspace.id }}
          </dd>
          <dt>Name</dt>
          <dd>
            {{ selectedWorkspace.name }}
          </dd>

          <dt>Snapshot Version</dt>
          <dd>
            {{ selectedWorkspace.snapshotVersion }}
          </dd>

          <dt class="font-bold">Component Concurrency Limit</dt>
          <dd>
            {{ selectedWorkspace.componentConcurrencyLimit ?? "default" }}
            <VButton class="m-1" @click="openModal">Set</VButton>
          </dd>
        </dl>

        <Stack v-if="workspaceUsers.length">
          <h3 class="font-bold text-sm">USERS</h3>
          <div
            v-for="user in workspaceUsers"
            :key="user.id"
            class="m-1 text-sm"
          >
            {{ user.name }} &lt;{{ user.email }}&gt;
          </div>
        </Stack>

        <dl
          v-if="selectedChangeSetId && selectedChangeSet"
          class="p-3 max-w-xs overflow-hidden"
        >
          <dt>Change Set Id</dt>
          <dd>
            {{ selectedChangeSet.id }}
          </dd>
          <dt>Name</dt>
          <dd>
            {{ selectedChangeSet.name }}
          </dd>
          <dt>Status</dt>
          <dd>
            {{ selectedChangeSet.status }}
          </dd>
          <dt>Snapshot Address (blake3 hash of contents)</dt>
          <dd>
            {{ selectedChangeSet.workspaceSnapshotAddress }}
          </dd>
        </dl>
      </div>

      <Modal ref="concurrencyModalRef" :title="concurrencyModalTitle">
        <Stack>
          <VormInput
            v-model="editingConcurrencyLimit"
            placeholder="blank is default"
            label="Enter a component concurrency limit for this workspace, blank for default"
            @keydown.enter="setConcurrencyLimit()"
          />
          <VButton
            :requestStatus="setConcurrencyLimitReqStatus"
            @click="setConcurrencyLimit()"
            >Set</VButton
          >
        </Stack>
      </Modal>

      <ValidateSnapshot v-if="showPanel === 'validate-snapshot'" />
      <MigrateConnections v-else-if="showPanel === 'migrate-connections'" />
    </template>
  </Stack>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import {
  Modal,
  Stack,
  VormInput,
  VButton,
  LoadStatus,
  useModal,
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { WorkspaceUser } from "@/store/auth.store";
import {
  AdminWorkspace,
  AdminChangeSet,
  useAdminStore,
} from "@/store/admin.store";
import { useWorkspacesStore, WorkspacePk } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import ValidateSnapshot from "@/components/AdminDashboard/ValidateSnapshot.vue";
import MigrateConnections from "@/components/AdminDashboard/MigrateConnections.vue";

const adminStore = useAdminStore();
const workspacesStore = useWorkspacesStore();

const searchWorkspacesReqStatus =
  adminStore.getRequestStatus("SEARCH_WORKSPACES");
const setConcurrencyLimitReqStatus = adminStore.getRequestStatus(
  "SET_CONCURRENCY_LIMIT",
);

const workspaceUsers = ref<WorkspaceUser[]>([]);
const workspaceChangeSets = ref<{ [key: string]: AdminChangeSet }>({});
const editingConcurrencyLimit = ref<string | null>(null);

const isGettingSnapshot = ref<boolean>(false);
const isSettingSnapshot = ref<boolean>(false);

const isGettingCasData = ref<boolean>(false);
const isUploadingCasData = ref<boolean>(false);

const lastUploadedSnapshotAddress = ref<string | null>(null);

const concurrencyModalRef = ref<InstanceType<typeof Modal>>();
const concurrencyModalTitle = computed(() =>
  selectedWorkspace.value
    ? `Set concurrency limit for ${selectedWorkspace.value.name} (${selectedWorkspace.value.id})`
    : "No workspace selected",
);

const { open: openModal, close: closeModal } = useModal(concurrencyModalRef);

const applyFilter = <T extends object>(
  things: { [key: string]: T },
  filter?: string | null,
): T[] => {
  const lowerCaseFilter = filter?.toLocaleLowerCase();
  return Object.values(things).filter((thing) =>
    lowerCaseFilter
      ? JSON.stringify(Object.values(thing))
          .toLocaleLowerCase()
          .includes(lowerCaseFilter)
      : true,
  );
};

const workspacesFilter = ref<string | null>(null);
const searchWorkspaces = async (filter?: string | null) => {
  const result = await adminStore.SEARCH_WORKSPACES(filter ?? undefined);
  if (result?.result.success) {
    filteredWorkspaces.value = result.result.data.workspaces;
  } else {
    filteredWorkspaces.value = [];
  }
};
// Start out searching for the current workspace
searchWorkspaces(workspacesStore.selectedWorkspacePk);
const filteredWorkspaces = ref<AdminWorkspace[]>([]);
const selectedWorkspaceId = ref<string | null>(null);
// When filteredWorkspaces changes, make sure selectedWorkspaceId is one of them
watch(filteredWorkspaces, (filteredWorkspaces) => {
  if (
    !(
      selectedWorkspaceId.value &&
      selectedWorkspaceId.value in filteredWorkspaces
    )
  ) {
    // Select the first workspace, unless the selected workspace is one of the results already
    selectedWorkspaceId.value = filteredWorkspaces[0]?.id ?? null;
  }
});
const selectedWorkspace = computed(() =>
  selectedWorkspaceId.value
    ? filteredWorkspaces.value.find(
        (workspace) => workspace.id === selectedWorkspaceId.value,
      )
    : undefined,
);
watch(selectedWorkspaceId, async (currentWorkspaceId) => {
  if (currentWorkspaceId) {
    editingConcurrencyLimit.value = selectedWorkspace.value
      ?.componentConcurrencyLimit
      ? String(selectedWorkspace.value.componentConcurrencyLimit)
      : null;
    await fetchChangeSets(currentWorkspaceId);
    await fetchUsers(currentWorkspaceId);
  }
});

const changeSetsFilter = ref<string | null>(null);
// Sort open change sets first

const ACTIVE_CHANGE_SET_STATUS = [
  "Open",
  "Approved",
  "NeedsApproval",
  "NeedsAbanadonApproval",
  "Rejected",
] as const;
const fetchChangeSets = async (workspaceId: string) => {
  const result = await adminStore.LIST_CHANGE_SETS(workspaceId);
  if (result?.result.success) {
    workspaceChangeSets.value = Object.fromEntries(
      Object.entries(result.result.data.changeSets).sort((a, b) => {
        // Sort HEAD first
        if (!a[1].baseChangeSetId !== !b[1].baseChangeSetId) {
          return !a[1].baseChangeSetId ? -1 : 1;
        }
        if (
          a[1].status in ACTIVE_CHANGE_SET_STATUS !==
          b[1].status in ACTIVE_CHANGE_SET_STATUS
        ) {
          return a[1].status in ACTIVE_CHANGE_SET_STATUS ? -1 : 1;
        }
        return a[1].name.localeCompare(b[1].name);
      }),
    );
  } else {
    workspaceChangeSets.value = {};
  }
};
const listChangeSetsReqStatus = computed(() =>
  adminStore.getRequestStatus("LIST_CHANGE_SETS", selectedWorkspaceId.value),
);
const filteredChangeSets = computed(() =>
  selectedWorkspaceId.value
    ? applyFilter(workspaceChangeSets.value ?? {}, changeSetsFilter.value)
    : [],
);
const selectedChangeSetId = ref<string | null>(null);
// When filteredChangeSets changes, make sure selectedChangeSetId is one of them
watch(filteredChangeSets, (filteredChangeSets) => {
  if (
    !(
      selectedChangeSetId.value &&
      selectedChangeSetId.value in filteredChangeSets
    )
  ) {
    // Select the first workspace, unless the selected workspace is one of the results already
    selectedChangeSetId.value = filteredChangeSets[0]?.id ?? null;
  }
});
const selectedChangeSet = computed(() =>
  selectedWorkspaceId.value && selectedChangeSetId.value
    ? workspaceChangeSets.value?.[selectedChangeSetId.value]
    : undefined,
);
if (
  !(
    selectedWorkspaceId.value &&
    selectedWorkspaceId.value in filteredWorkspaces.value
  )
) {
  // Select the first workspace, unless the selected workspace is one of the results already
  selectedWorkspaceId.value = filteredWorkspaces.value[0]?.id ?? null;
}

const fetchUsers = async (workspaceId: string) => {
  const result = await adminStore.LIST_WORKSPACE_USERS(workspaceId);
  if (result?.result.success) {
    workspaceUsers.value = result.result.data.users;
  } else {
    workspaceUsers.value = [];
  }
};

const getSnapshot = async (workspaceId: string, changeSetId: string) => {
  isGettingSnapshot.value = true;
  try {
    const result = await adminStore.GET_SNAPSHOT(workspaceId, changeSetId);
    if (result.result.success) {
      const bytes = Buffer.from(result.result.data, "base64");
      const blob = new Blob([bytes], { type: "application/octet-stream" });

      const fileHandle = await window.showSaveFilePicker({
        suggestedName: `${changeSetId}.snapshot`,
        types: [
          {
            description: `Workspace Snapshot for change set ${changeSetId}`,
            accept: {
              "application/octet-stream": [".snapshot"],
            },
          },
        ],
      });

      const writable = await fileHandle.createWritable();
      await writable.write(blob);
      await writable.close();
    }
  } finally {
    isGettingSnapshot.value = false;
  }
};

const setSnapshot = async (workspaceId: string, changeSetId: string) => {
  isSettingSnapshot.value = true;
  try {
    const [fileHandle] = await window.showOpenFilePicker({
      types: [
        {
          description: `Workspace Snapshot for change set ${changeSetId}`,
          accept: {
            "application/octet-stream": [".snapshot"],
          },
        },
      ],
    });

    const fileData = await fileHandle.getFile();
    const result = await adminStore.SET_SNAPSHOT(
      workspaceId,
      changeSetId,
      fileData,
    );
    if (result.result.success) {
      lastUploadedSnapshotAddress.value =
        result.result.data.workspaceSnapshotAddress;
    }
    if (selectedWorkspaceId.value) {
      await fetchChangeSets(selectedWorkspaceId.value);
    }
  } finally {
    isSettingSnapshot.value = false;
  }
};

const uploadCasData = async (workspaceId: string, changeSetId: string) => {
  isUploadingCasData.value = true;
  try {
    const [fileHandle] = await window.showOpenFilePicker({
      types: [
        {
          description: `Cas data map for change set ${changeSetId}`,
          accept: {
            "application/octet-stream": [".cas"],
          },
        },
      ],
    });

    const fileData = await fileHandle.getFile();
    await adminStore.UPLOAD_CAS_DATA(workspaceId, changeSetId, fileData);
  } finally {
    isUploadingCasData.value = false;
  }
};

const getCasData = async (workspaceId: string, changeSetId: string) => {
  isGettingCasData.value = true;
  try {
    const result = await adminStore.GET_CAS_DATA(workspaceId, changeSetId);
    if (result.result.success) {
      const bytes = Buffer.from(result.result.data, "base64");
      const blob = new Blob([bytes], { type: "application/octet-stream" });

      const fileHandle = await window.showSaveFilePicker({
        suggestedName: `${changeSetId}.cas`,
        types: [
          {
            description: `Cas Data map for change set ${changeSetId}`,
            accept: {
              "application/octet-stream": [".cas"],
            },
          },
        ],
      });

      const writable = await fileHandle.createWritable();
      await writable.write(blob);
      await writable.close();
    }
  } finally {
    isGettingCasData.value = false;
  }
};

const setConcurrencyLimit = async () => {
  const componentConcurrencyLimit = editingConcurrencyLimit.value
    ? parseInt(editingConcurrencyLimit.value)
    : undefined;
  if (selectedWorkspaceId.value) {
    const result = await adminStore.SET_CONCURRENCY_LIMIT(
      selectedWorkspaceId.value,
      componentConcurrencyLimit,
    );
    if (result.result.success) {
      const newLimit = result.result.data.concurrencyLimit;
      filteredWorkspaces.value = filteredWorkspaces.value.map((workspace) => {
        if (workspace.id === selectedWorkspaceId.value) {
          return {
            ...workspace,
            componentConcurrencyLimit: newLimit,
          };
        }
        return workspace;
      });
      closeModal();
    }
  }
};

const showPanel = ref<"validate-snapshot" | "migrate-connections" | null>(null);
const migrateConnectionsRequestStatus = adminStore.getRequestStatus(
  "MIGRATE_CONNECTIONS",
);
function migrateConnections(
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  options?: { dryRun?: boolean },
) {
  adminStore.MIGRATE_CONNECTIONS(workspaceId, changeSetId, options);
  showPanel.value = "migrate-connections";
}

const validateSnapshotRequestStatus =
  adminStore.getRequestStatus("VALIDATE_SNAPSHOT");

function validateSnapshot(workspaceId: WorkspacePk, changeSetId: ChangeSetId) {
  adminStore.VALIDATE_SNAPSHOT(workspaceId, changeSetId);
  showPanel.value = "validate-snapshot";
}
</script>
