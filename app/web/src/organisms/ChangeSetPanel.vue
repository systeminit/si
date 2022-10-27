<template>
  <div>
    <section class="p-sm">
      <div class="flex items-center gap-x-xs">
        <VormInput
          class="flex-grow"
          type="dropdown"
          :model-value="selectedChangeSetId"
          label="Change Set"
          placeholder="no changeset selected"
          :options="changeSetDropdownOptions"
          @update:model-value="onSelectChangeSet"
        >
          <VormInputOption value="NEW">
            - Create new change set -
          </VormInputOption>
        </VormInput>

        <VormInput type="container">
          <VButton2 icon="git-merge" size="sm" @click="applyChangeSet">
            Apply
          </VButton2>
        </VormInput>
      </div>
    </section>
    <Modal
      :open="showDialog === 'create'"
      size="sm"
      hide-top-close-button
      type="custom"
      @close="onCloseCreateDialog"
    >
      <template #title>Create Change Set</template>
      <template #content>
        <div>
          <p class="pb-2 type-regular-sm">
            Modeling a configuration or extending SI happens within
            <b>Change Sets</b>. Think of these like light-weight branches,
            allowing you to experiment freely without risk of impacting
            production systems.
          </p>
          <p class="pb-2 type-regular-sm">
            Please give your <b>Change Set</b> a name below, and click the
            Create button.
          </p>
        </div>
        <div class="pt-2">
          <VormInput
            v-model="createChangeSetName"
            label="Change set name"
            required
            required-message="Please choose a name for your change set!"
          />
        </div>
      </template>
      <template #buttons>
        <div class="flex flex-row-reverse gap-sm">
          <VButton2
            :disabled="!createChangeSetName"
            tone="success"
            icon="plus-circle"
            label="Create"
            class="flex-grow"
            @click="onCreateChangeSet"
          />
          <VButton2
            tone="destructive"
            variant="ghost"
            icon="x-circle"
            label="Cancel"
            @click="onCloseCreateDialog"
          />
        </div>
      </template>
    </Modal>

    <Modal
      :open="showDialog === 'select'"
      size="sm"
      hide-top-close-button
      type="custom"
      @close="onCloseSelectDialog"
    >
      <template #title>Select Change Set</template>
      <template #content>
        <div class="type-regular-sm pt-2">
          <p>
            Select the Change Set you would like to resume working in, or select
            <b>- new -</b> to create a new Change Set.
          </p>
        </div>
        <Stack>
          <VormInput
            type="dropdown"
            :model-value="selectedChangeSetId"
            :options="changeSetDropdownOptions"
            placeholder="Select an existing change set"
            class="flex-grow"
            @update:model-value="onSelectChangeSet"
          />
          <Divider label="or" />
          <VButton2 icon="plus-circle" @click="createNewChangeset">
            Create a new change set
          </VButton2>
        </Stack>
      </template>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import _ from "lodash";
import { useRoute, useRouter } from "vue-router";
import { useChangeSetsStore } from "@/store/change_sets.store";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import VormInputOption from "@/ui-lib/forms/VormInputOption.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import Divider from "@/ui-lib/layout/Divider.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import Modal from "@/ui-lib/Modal.vue";

const workspacesStore = useWorkspacesStore();
const selectedWorkspaceId = computed(() => workspacesStore.selectedWorkspaceId);

const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);

const changeSetDropdownOptions = computed(() =>
  _.map(openChangeSets.value, (cs) => ({ value: cs.id, label: cs.name })),
);

const router = useRouter();
const route = useRoute();

// Determines whether or not to display a dialog
const showDialog = ref<false | "create" | "select">(false);

// The name for a new change set
const createChangeSetName = ref("");

function onSelectChangeSet(newVal: number | "NEW") {
  if (newVal === "NEW") {
    showDialog.value = "create";
  } else if (newVal && route.name) {
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: newVal,
      },
    });
    showDialog.value = false;
  }
}
async function onCreateChangeSet() {
  if (!createChangeSetName.value.trim()) return;

  const createReq = await changeSetsStore.CREATE_CHANGE_SET(
    createChangeSetName.value,
  );
  if (createReq.result.success) {
    // reusing above to navigate to new change set... will probably clean this all up later
    onSelectChangeSet(createReq.result.data.changeSet.id);
  }
}

// Saves the current edit session and then applies the current change set
const applyChangeSet = async () => {
  const applyReq = await changeSetsStore.APPLY_CHANGE_SET();
  if (applyReq.result.success) await navigateToFixMode();
};

const createNewChangeset = () => {
  showDialog.value = "create";
};

watch(
  openChangeSets,
  () => {
    if (!openChangeSets.value.length) {
      showDialog.value = "create";
    } else if (!selectedChangeSetId.value) {
      showDialog.value = "select";
    }
  },
  { immediate: true },
);

// Navigates to the workspace fix page
const navigateToFixMode = async () => {
  if (selectedWorkspaceId.value) {
    await router.push({
      name: "workspace-fix",
      path: "/w/:workspaceId/r",
      params: { workspaceId: selectedWorkspaceId.value },
    });
  } else {
    // Fallback to the workspace list page in the case we can't yet determine
    // the current workspace (likely due to an observable race).
    await router.push({ name: "workspace-index" });
  }
};

function onCloseCreateDialog() {
  showDialog.value = false;
  if (!selectedChangeSetId.value) navigateToFixMode();
}
function onCloseSelectDialog() {
  showDialog.value = false;
  if (!selectedChangeSetId.value) navigateToFixMode();
}
</script>
