<template>
  <div class="flex flex-row gap-xs items-end flex-1 min-w-[172px] max-w-fit">
    <label class="flex flex-col flex-1 min-w-0 max-w-fit">
      <div
        class="text-[11px] mt-[1px] mb-[5px] capsize font-medium text-neutral-300 whitespace-nowrap"
      >
        CHANGE SET:
      </div>
      <DropdownMenuButton
        ref="dropdownMenuRef"
        v-model="selectedChangeSetId"
        :options="changeSetSearchFilteredOptions"
        :search="
          changeSetDropdownOptions.length > DEFAULT_DROPDOWN_SEARCH_THRESHOLD
        "
        placeholder="-- select a change set --"
        checkable
        variant="navbar"
        :enableSecondaryAction="calculateShowSecondaryAction"
        :sizeClass="tw`h-[28px]`"
        secondaryActionIcon="edit2"
        @select="onSelectChangeSet"
        @secondaryAction="openRenameModal"
      >
        <template #afterOptions>
          <DropdownMenuItem
            label="Create New Change Set"
            icon="plus"
            disableCheckable
            @select="
              () => {
                onSelectChangeSet('NEW');
              }
            "
          />
        </template>
        <DropdownMenuItem
          v-if="changeSetSearchFilteredOptions.length === 0"
          label="No Change Sets Match Your Search"
          header
        />
      </DropdownMenuButton>
    </label>

    <NewButton
      v-tooltip="{
        content: 'Create Change Set',
      }"
      data-testid="create-change-set-button"
      icon="git-branch-plus"
      tone="action"
      class="flex-none"
      @click="openCreateModal"
    />

    <NewButton
      v-tooltip="{
        content: 'Abandon Change Set',
      }"
      data-testid="abandon-change-set-button"
      :disabled="
        !selectedChangeSetName ||
        changeSetsStore.headSelected ||
        changeSetsStore.creatingChangeSet ||
        changeSetsStore.selectedChangeSet?.status ===
          ChangeSetStatus.NeedsApproval
      "
      icon="trash"
      tone="destructive"
      class="flex-none"
      @click="openAbandonConfirmationModal"
    />

    <NewButton
      v-tooltip="{
        content: 'Open Remote Shell',
      }"
      data-testid="remote-shell-button"
      :disabled="
        !selectedChangeSetName ||
        changeSetsStore.headSelected ||
        changeSetsStore.creatingChangeSet ||
        changeSetsStore.selectedChangeSet?.status ===
          ChangeSetStatus.NeedsApproval
      "
      icon="command"
      tone="action"
      class="flex-none"
      @click="openRemoteShellModal"
    />

    <Modal ref="createModalRef" title="Create Change Set">
      <form @submit.prevent="onCreateChangeSet">
        <Stack>
          <p>
            Modeling a configuration or extending SI happens within
            <b>Change Sets</b>. Think of these like light-weight branches,
            allowing you to experiment freely without risk of impacting
            production systems.
          </p>
          <p>
            Please give your <b>Change Set</b> a name below, and click the
            Create button.
          </p>
          <VormInput
            v-model="createChangeSetName"
            :regex="CHANGE_SET_NAME_REGEX"
            label="Change set name"
            regexMessage="You cannot name a change set 'HEAD' - please choose another name."
            required
            requiredMessage="Please choose a name for your change set!"
            @enterPressed="onCreateChangeSet"
          />
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="validationState.isError"
              :requestStatus="createChangeSetReqStatus"
              class="flex-grow"
              icon="plus-circle"
              label="Create change set"
              loadingText="Creating Change Set"
              submit
              tone="success"
            />
          </div>
        </Stack>
      </form>
    </Modal>
    <AbandonChangeSetModal ref="abandonModalRef" />
    <ChangesetRenameModal v-if="enableChangesetRename" ref="renameModalRef" />
    <RemoteShellTerminal ref="remoteShellModalRef" />
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
  VButton,
  VormInput,
  Stack,
  Modal,
  useValidatedInputGroup,
  DropdownMenuButton,
  DropdownMenuItem,
  DEFAULT_DROPDOWN_SEARCH_THRESHOLD,
  NewButton,
} from "@si/vue-lib/design-system";
import { tw } from "@si/vue-lib";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import AbandonChangeSetModal from "@/components/AbandonChangeSetModal.vue";
import RemoteShellTerminal from "@/components/RemoteShellTerminal.vue";
import { reset } from "@/newhotness/logic_composables/navigation_stack";
import ChangesetRenameModal from "@/components/ChangesetRenameModal.vue";
import * as heimdall from "../../../store/realtime/heimdall";

const CHANGE_SET_NAME_REGEX = /^(?!head).*$/i;

const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenuButton>>();

const changeSetDropdownOptions = computed(() => [
  ..._.map(openChangeSets.value, (cs) => ({
    value: cs.id,
    label: cs.name,
  })),
  // { value: "NEW", label: "+ Create new change set" },
]);

const changeSetSearchFilteredOptions = computed(() => {
  const searchString = dropdownMenuRef.value?.searchString;

  if (!searchString || searchString === "") {
    return changeSetDropdownOptions.value;
  }

  return changeSetDropdownOptions.value.filter(
    (option) =>
      option.label.toLocaleLowerCase().includes(searchString) ||
      option.value.toLocaleLowerCase().includes(searchString),
  );
});

const router = useRouter();
const route = useRoute();

// NOTE(victor): The changeset modal uses the new api lib, so we're not enabling the rename on the old ui
// TODO: Remove this when the old ui is deprecated
const enableChangesetRename = computed(() =>
  route.name?.toString().startsWith("new-hotness"),
);

const calculateShowSecondaryAction = (option: { label: string }) => {
  return enableChangesetRename.value && option.label.toUpperCase() !== "HEAD";
};

const abandonModalRef = ref<InstanceType<typeof AbandonChangeSetModal> | null>(
  null,
);
const openAbandonConfirmationModal = () => {
  abandonModalRef.value?.open();
};

const remoteShellModalRef = ref<InstanceType<typeof RemoteShellTerminal> | null>(
  null,
);
const openRemoteShellModal = () => {
  remoteShellModalRef.value?.open();
};

const renameModalRef = ref<InstanceType<typeof ChangesetRenameModal> | null>(
  null,
);
function openRenameModal(option: { value: string; label: string }) {
  renameModalRef.value?.open(option.value, option.label);
}

const createModalRef = ref<InstanceType<typeof Modal>>();

const changeSetsReqStatus =
  changeSetsStore.getRequestStatus("FETCH_CHANGE_SETS");

const checkFirstLoad = () => {
  if (!changeSetsReqStatus.value.isSuccess || !createModalRef.value) return;

  const isFirstLoad = !window.localStorage.getItem("ran-first-load");
  window.localStorage.setItem("ran-first-load", "true");

  if (isFirstLoad) {
    createModalRef.value?.open();
  }
};

watch([changeSetsReqStatus], checkFirstLoad);

onMounted(() => {
  checkFirstLoad();
});

// We don't add a change set name
const createChangeSetName = ref("");

const { validationState, validationMethods } = useValidatedInputGroup();

const waitForChangeSetExists = (
  workspaceId: string,
  changeSetId: string,
): Promise<void> => {
  const INTERVAL_MS = 50;
  const MAX_WAIT_IN_SEC = 10;
  const MAX_RETRIES = (MAX_WAIT_IN_SEC * 1000) / INTERVAL_MS;

  return new Promise((resolve, reject) => {
    let retry = 0;
    const interval = setInterval(async () => {
      if (retry >= MAX_RETRIES) {
        clearInterval(interval);
        reject();
      }

      if (await heimdall.changeSetExists(workspaceId, changeSetId)) {
        clearInterval(interval);
        resolve();
      }
      retry += 1;
    }, INTERVAL_MS);
  });
};

async function onSelectChangeSet(newVal: string) {
  if (newVal === "NEW") {
    createModalRef.value?.open();
    return;
  }

  if (newVal && route.name) {
    // keep everything in the current route except the change set id
    // note - we use push here, so there is a new browser history entry

    if (
      route.name?.toString().startsWith("new-hotness") &&
      typeof route.params.workspacePk === "string"
    ) {
      const workspaceId = route.params.workspacePk;
      await waitForChangeSetExists(workspaceId, newVal);
    }

    const name = route.name;
    await router.push({
      name,
      params: {
        ...route.params,
        changeSetId: newVal,
      },
      query: route.query,
    });
    reset();
  }
}

async function onCreateChangeSet() {
  if (validationMethods.hasError()) return;

  const createReq = await changeSetsStore.CREATE_CHANGE_SET(
    createChangeSetName.value,
  );

  if (createReq.result.success && createReq.result.data?.id) {
    const newChangeSetId = createReq.result.data.id;
    onSelectChangeSet(newChangeSetId);
    createModalRef.value?.close();
  }
}

const createChangeSetReqStatus =
  changeSetsStore.getRequestStatus("CREATE_CHANGE_SET");

function openCreateModal() {
  if (createModalRef.value?.isOpen) return;
  // Reset the name on new open of the modal
  createChangeSetName.value = "";
  createModalRef.value?.open();
}

defineExpose({ openCreateModal });
</script>
