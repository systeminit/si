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
        @select="onSelectChangeSet"
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

    <VButton
      v-tooltip="{
        content: 'Create Change Set',
      }"
      icon="git-branch-plus"
      size="sm"
      tone="action"
      variant="ghost"
      class="flex-none"
      @click="openCreateModal"
    />

    <VButton
      v-tooltip="{
        content: 'Abandon Change Set',
      }"
      :disabled="
        !selectedChangeSetName ||
        changeSetsStore.headSelected ||
        changeSetsStore.creatingChangeSet ||
        changeSetsStore.selectedChangeSet?.status ===
          ChangeSetStatus.NeedsApproval
      "
      icon="trash"
      size="sm"
      tone="action"
      variant="ghost"
      class="flex-none"
      @click="openAbandonConfirmationModal"
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
  </div>
</template>

<script lang="ts" setup>
import { onMounted, computed, ref, watch } from "vue";
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
} from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { ChangeSetStatus } from "@/api/sdf/dal/change_set";
import AbandonChangeSetModal from "@/components/AbandonChangeSetModal.vue";
import { reset } from "@/newhotness/logic_composables/navigation_stack";
import * as heimdall from "../../../store/realtime/heimdall";

const CHANGE_SET_NAME_REGEX = /^(?!head).*$/i;

const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenuButton>>();

const changeSetDropdownOptions = computed(() => {
  const options = [
    ..._.map(openChangeSets.value, (cs) => ({ value: cs.id, label: cs.name })),
    // { value: "NEW", label: "+ Create new change set" },
  ];
  return options;
});

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

const abandonModalRef = ref<InstanceType<typeof AbandonChangeSetModal> | null>(
  null,
);
const openAbandonConfirmationModal = () => {
  abandonModalRef.value?.open();
};

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

async function onSelectChangeSet(newVal: string) {
  if (newVal === "NEW") {
    createModalRef.value?.open();
    return;
  }

  if (newVal && route.name) {
    // keep everything in the current route except the change set id
    // note - we use push here, so there is a new browser history entry
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

  if (createReq.result.success) {
    // reusing above to navigate to new change set... will probably clean this all up later
    const newChangeSetId = createReq.result.data.changeSet.id;
    heimdall.muspelheimStatuses.value[newChangeSetId] = false;
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
