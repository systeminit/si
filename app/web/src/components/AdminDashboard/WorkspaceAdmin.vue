<template>
  <Stack class="max-w-xl">
    <h2 class="font-bold text-xl">WORKSPACES</h2>
    <VormInput
      v-model="workspacesFilter"
      label="type here to search for a workspace by id or name, or change set id or name, or user id, name, or email (50 results max). then press ENTER"
      @keydown.enter="searchWorkspaces(workspacesFilter)"
    />
    <LoadingMessage :requestStatus="searchWorkspacesReqStatus" />
    <p>{{ filteredWorkspaces?.length ?? 0 }} found</p>
    <select
      v-model="selectedWorkspaceId"
      class="text-neutral-900 dark:text-neutral-200 dark:bg-neutral-900 bg-neutral-100"
    >
      <option
        v-for="workspace in filteredWorkspaces"
        :key="workspace.id"
        :value="workspace.id"
      >
        {{ workspace.name }} ({{ workspace.id }})
      </option>
    </select>
    <Stack v-if="selectedWorkspace">
      <dl class="p-3">
        <dt class="p-1 font-bold">Workspace Id</dt>
        <dd class="m-3 pl-2">
          <pre>{{ selectedWorkspace.id }}</pre>
        </dd>
        <dt class="p-1 font-bold">Name</dt>
        <dd class="m-3 pl-2">
          <pre>{{ selectedWorkspace.name }}</pre>
        </dd>

        <dt class="p-1 font-bold">Snapshot Version</dt>
        <dd class="m-3 pl-2">
          <pre>{{ selectedWorkspace.snapshotVersion }}</pre>
        </dd>

        <dt class="font-bold">Component Concurrency Limit</dt>
        <dd class="m-3 pl-2">
          <pre>{{
            selectedWorkspace.componentConcurrencyLimit ?? "default"
          }}</pre>
          <VButton class="m-1" @click="openModal">Set</VButton>
        </dd>
      </dl>

      <Stack v-if="workspaceUsers.length">
        <h3 class="font-bold text-sm">USERS</h3>
        <div v-for="user in workspaceUsers" :key="user.id" class="m-1 text-sm">
          {{ user.name }} &lt;{{ user.email }}&gt;
        </div>
      </Stack>

      <Stack>
        <h3 class="font-bold text-md mb-3">CHANGE SETS</h3>
        <VormInput
          v-model="changeSetsFilter"
          label="type here to filter the change set list (e.g., type 'open' to see only open change sets)"
        />
        <LoadingMessage :requestStatus="listChangeSetsReqStatus.value" />
        <select
          v-if="selectedWorkspaceId && filteredChangeSets?.length"
          v-model="selectedChangeSetId"
          class="text-neutral-900 dark:text-neutral-200 dark:bg-neutral-900 bg-neutral-100"
        >
          <option
            v-for="changeSet in filteredChangeSets"
            :key="changeSet.id"
            :value="changeSet.id"
          >
            {{ changeSet.name }} ({{ changeSet.id }}) --
            {{ changeSet.status }}
            <p v-if="changeSet.id === selectedWorkspace?.defaultChangeSetId">
              *
            </p>
          </option>
        </select>
        <p v-else-if="!selectedWorkspaceId">No workspace selected...</p>
        <p v-else-if="listChangeSetsReqStatus.value?.isPending">
          Loading change sets...
        </p>
        <p v-else-if="filteredChangeSets?.length === 0">
          No change sets for workspace...
        </p>
        <Stack
          v-if="selectedChangeSetId && selectedChangeSet && selectedWorkspaceId"
        >
          <dl class="p-3">
            <dt class="p-1 font-bold">Change Set Id</dt>
            <dd class="m-3 pl-2">
              <pre>{{ selectedChangeSet.id }}</pre>
            </dd>
            <dt class="p-1 font-bold">Name</dt>
            <dd class="m-3 pl-2">
              <pre>{{ selectedChangeSet.name }}</pre>
            </dd>
            <dt class="p-1 font-bold">Status</dt>
            <dd class="m-3 pl-2">
              <pre>{{ selectedChangeSet.status }}</pre>
            </dd>
            <dt class="p-1 font-bold">
              Snapshot Address (blake3 hash of contents)
            </dt>
            <dd class="m-3 pl-2">
              <pre>{{ selectedChangeSet.workspaceSnapshotAddress }}</pre>
            </dd>
          </dl>

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
      </Stack>
    </Stack>
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
  </Stack>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import {
  Modal,
  Stack,
  VormInput,
  VButton,
  LoadingMessage,
  useModal,
} from "@si/vue-lib/design-system";
import { WorkspaceUser } from "@/store/auth.store";
import {
  AdminWorkspace,
  AdminChangeSet,
  useAdminStore,
} from "@/store/admin.store";
import { WorkspacePk } from "@/store/workspaces.store";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { PanelKind } from "../Workspace/WorkspaceAdminDashboard.vue";

const emit = defineEmits<{
  showPanel: [PanelKind];
}>();

const adminStore = useAdminStore();

const searchWorkspacesReqStatus =
  adminStore.getRequestStatus("SEARCH_WORKSPACES");
const setConcurrencyLimitReqStatus = adminStore.getRequestStatus(
  "SET_CONCURRENCY_LIMIT",
);

const workspacesFilter = ref<string | null>(null);
const filteredWorkspaces = ref<AdminWorkspace[]>([]);
const selectedWorkspaceId = ref<string | null>(null);
const workspaceUsers = ref<WorkspaceUser[]>([]);
const workspaceChangeSets = ref<{ [key: string]: AdminChangeSet }>({});
const editingConcurrencyLimit = ref<string | null>(null);

const changeSetsFilter = ref<string | null>(null);
const selectedChangeSetId = ref<string | null>(null);

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

const selectedWorkspace = computed(() =>
  selectedWorkspaceId.value
    ? filteredWorkspaces.value.find(
        (workspace) => workspace.id === selectedWorkspaceId.value,
      )
    : undefined,
);

const selectedChangeSet = computed(() =>
  selectedWorkspaceId.value && selectedChangeSetId.value
    ? workspaceChangeSets.value?.[selectedChangeSetId.value]
    : undefined,
);

const fetchChangeSets = async (workspaceId: string) => {
  const result = await adminStore.LIST_CHANGE_SETS(workspaceId);
  if (result?.result.success) {
    workspaceChangeSets.value = result.result.data.changeSets;
  } else {
    workspaceChangeSets.value = {};
  }
};

const fetchUsers = async (workspaceId: string) => {
  const result = await adminStore.LIST_WORKSPACE_USERS(workspaceId);
  if (result?.result.success) {
    workspaceUsers.value = result.result.data.users;
  } else {
    workspaceUsers.value = [];
  }
};

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

const listChangeSetsReqStatus = computed(() =>
  adminStore.getRequestStatus("LIST_CHANGE_SETS", selectedWorkspaceId.value),
);

const searchWorkspaces = async (filter?: string | null) => {
  const result = await adminStore.SEARCH_WORKSPACES(filter ?? undefined);
  if (result?.result.success) {
    filteredWorkspaces.value = result.result.data.workspaces;
  } else {
    filteredWorkspaces.value = [];
  }
};

const filteredChangeSets = computed(() =>
  selectedWorkspaceId.value
    ? applyFilter(workspaceChangeSets.value ?? {}, changeSetsFilter.value)
    : [],
);

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

const migrateConnectionsRequestStatus = adminStore.getRequestStatus(
  "MIGRATE_CONNECTIONS",
);
function migrateConnections(
  workspaceId: WorkspacePk,
  changeSetId: ChangeSetId,
  options?: { dryRun?: boolean },
) {
  adminStore.MIGRATE_CONNECTIONS(workspaceId, changeSetId, options);
  emit("showPanel", "migrate-connections");
}

const validateSnapshotRequestStatus =
  adminStore.getRequestStatus("VALIDATE_SNAPSHOT");

function validateSnapshot(workspaceId: WorkspacePk, changeSetId: ChangeSetId) {
  adminStore.VALIDATE_SNAPSHOT(workspaceId, changeSetId);
  emit("showPanel", "validate-snapshot");
}
</script>
