<template>
  <div
    class="w-full h-full flex flex-col items-center relative overflow-hidden dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <Stack spacing="lg" class="p-10 w-full">
      <span class="flex flex-row mt-10 font-bold text-3xl"
        >Admin Dashboard</span
      >
      <Stack class="max-w-xl">
        <h2 class="font-bold text-lg">KILL FUNCTION EXECUTION</h2>
        <VormInput
          v-model="funcRunId"
          label="FuncRunId for function execution"
        />
        <div class="flex flex-row-reverse gap-sm">
          <VButton
            :disabled="!funcRunId"
            :requestStatus="killExecutionReqStatus"
            class="flex-grow"
            icon="plus-circle"
            label="Kill function execution"
            loadingText="Killing function execution"
            tone="success"
            @click="killExecution"
          />
        </div>
      </Stack>
      <Stack class="max-w-xl">
        <h2 class="font-bold text-xl">WORKSPACES</h2>
        <VormInput
          v-model="workspacesFilter"
          label="type here to filter the workspaces list"
        />
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
            <dt class="p-1 font-bold">Id</dt>
            <dd class="m-3 pl-2">
              <pre>{{ selectedWorkspace.id }}</pre>
            </dd>
            <dt class="p-1 font-bold">Name</dt>
            <dd class="m-3 pl-2">
              <pre>{{ selectedWorkspace.name }}</pre>
            </dd>

            <dt class="p-1 font-bold">Snapshot Version</dt>
            <dd class="m-3 pl-2">{{ selectedWorkspace.snapshotVersion }}</dd>

            <dt class="font-bold">Component Concurrency Limit</dt>
            <dd class="m-3 pl-2">
              {{ selectedWorkspace.componentConcurrencyLimit }}
            </dd>
          </dl>

          <Stack>
            <h3 class="font-bold text-md mb-3">CHANGE SETS</h3>
            <VormInput
              v-model="changeSetsFilter"
              label="type here to filter the change set list (e.g., type 'open' to see only open change sets)"
            />

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
                <p
                  v-if="
                    changeSet.id ===
                    adminStore.workspaces[selectedWorkspaceId]
                      ?.defaultChangeSetId
                  "
                >
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
              v-if="
                selectedChangeSetId && selectedChangeSet && selectedWorkspaceId
              "
            >
              <dl class="p-3">
                <dt class="p-1 font-bold">Id</dt>
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
                :loading="isSavingSnapshot"
                @click="getSnapshot(selectedWorkspaceId, selectedChangeSetId)"
                >Save snapshot to disk</VButton
              >
            </Stack>
          </Stack>
        </Stack>
      </Stack>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, ref, watch } from "vue";
import { Stack, VormInput, VButton } from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useAdminStore } from "@/store/admin.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const adminStore = useAdminStore();
const featureFlagStore = useFeatureFlagsStore();

const router = useRouter();
onBeforeMount(async () => {
  if (!featureFlagStore.ADMIN_PANEL_ACCESS) {
    await router.push({ name: "workspace-single" });
  }
});

const killExecutionReqStatus = adminStore.getRequestStatus("KILL_EXECUTION");

const funcRunId = ref<string | null>(null);

const workspacesFilter = ref<string | null>(null);
const selectedWorkspaceId = ref<string | null>(null);

const changeSetsFilter = ref<string | null>(null);
const selectedChangeSetId = ref<string | null>(null);

const isSavingSnapshot = ref<boolean>(false);

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
    ? adminStore.workspaces[selectedWorkspaceId.value]
    : undefined,
);

const selectedChangeSet = computed(() =>
  selectedWorkspaceId.value && selectedChangeSetId.value
    ? adminStore.changeSetsByWorkspaceId[selectedWorkspaceId.value]?.[
        selectedChangeSetId.value
      ]
    : undefined,
);

watch(selectedWorkspaceId, async (currentWorkspaceId) => {
  if (currentWorkspaceId) {
    await adminStore.LIST_CHANGE_SETS(currentWorkspaceId);
  }
});

const listChangeSetsReqStatus = computed(() =>
  adminStore.getRequestStatus("LIST_CHANGE_SETS", selectedWorkspaceId.value),
);

const filteredWorkspaces = computed(() =>
  applyFilter(adminStore.workspaces, workspacesFilter.value),
);

const filteredChangeSets = computed(() =>
  selectedWorkspaceId.value
    ? applyFilter(
        adminStore.changeSetsByWorkspaceId[selectedWorkspaceId.value] ?? {},
        changeSetsFilter.value,
      )
    : [],
);

const killExecution = () => {
  if (funcRunId.value) {
    adminStore.KILL_EXECUTION(funcRunId.value);
  }
};

const getSnapshot = async (workspaceId: string, changeSetId: string) => {
  isSavingSnapshot.value = true;
  try {
    const result = await adminStore.FETCH_SNAPSHOT(workspaceId, changeSetId);
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
    isSavingSnapshot.value = false;
  }
};
</script>
