<template>
  <div class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0">
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
      </div>
    </section>

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
            label="Change set name"
            required
            required-message="Please choose a name for your change set!"
          />
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="validationState.isError"
              tone="success"
              icon="plus-circle"
              label="Create change set"
              loading-text="Creating Change Set"
              :request-status="createChangeSetReqStatus"
              class="flex-grow"
              submit
            />
          </div>
        </Stack>
      </form>
    </Modal>

    <Modal ref="selectModalRef" no-exit title="Select Change Set">
      <Stack>
        <p>
          Select the Change Set you would like to resume working in, or select
          <b>- new -</b> to create a new Change Set.
        </p>
        <VormInput
          type="dropdown"
          no-label
          :model-value="selectedChangeSetId"
          :options="changeSetDropdownOptions"
          placeholder="Select an existing change set"
          class="flex-grow"
          @update:model-value="onSelectChangeSet"
        />
        <Divider label="or" />
        <VButton icon="plus-circle" @click="openCreateModal">
          Create a new change set
        </VButton>
      </Stack>
    </Modal>

    <Wipe ref="wipeRef">
      <template #duringWipe>
        <VButton
          icon="loader"
          size="md"
          label="Merging"
          class="!bg-action-600"
        />
      </template>
      <template #afterWipe>
        <div
          v-if="changeSetMergeStatus.isPending || wipeRef?.state === 'running'"
          class="gap-2 items-center flex flex-row p-xl min-w-0 w-full justify-center"
        >
          <Icon name="loader" size="2xl" />
          <span class="text-3xl italic truncate">
            Merging Change Set<template v-if="selectedChangeSetName">
              "{{ selectedChangeSetName }}"
            </template>
          </span>
        </div>
      </template>
    </Wipe>
  </div>
</template>

<script lang="ts" setup>
import { computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
  VButton,
  Icon,
  VormInput,
  VormInputOption,
  Divider,
  Stack,
  Modal,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import Wipe from "./Wipe.vue";

const wipeRef = ref<InstanceType<typeof Wipe>>();

const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const changeSetDropdownOptions = computed(() =>
  _.map(openChangeSets.value, (cs) => ({ value: cs.id, label: cs.name })),
);

const router = useRouter();
const route = useRoute();

const createModalRef = ref<InstanceType<typeof Modal>>();
const selectModalRef = ref<InstanceType<typeof Modal>>();

// The name for a new change set
const createChangeSetName = ref("");

const { validationState, validationMethods } = useValidatedInputGroup();

function onSelectChangeSet(newVal: string | "NEW") {
  if (newVal === "NEW") {
    openCreateModal();
  } else if (newVal && route.name) {
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: newVal,
      },
    });
  }
}

async function onCreateChangeSet() {
  if (validationMethods.hasError()) return;

  const createReq = await changeSetsStore.CREATE_CHANGE_SET(
    createChangeSetName.value,
  );
  if (createReq.result.success) {
    // reusing above to navigate to new change set... will probably clean this all up later
    onSelectChangeSet(createReq.result.data.changeSet.pk);
  }
}

const createChangeSetReqStatus =
  changeSetsStore.getRequestStatus("CREATE_CHANGE_SET");

const changeSetMergeStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

function getGeneratedChangesetName() {
  // TODO: do we want to autogenerate names when not in dev? Maybe toggle-able setting?
  if (!import.meta.env.DEV) return "";
  let latestNum = 0;
  _.each(changeSetsStore.allChangeSets, (cs) => {
    const labelNum = parseInt(cs.name.split(" ").pop() || "");
    if (!_.isNaN(labelNum) && labelNum > latestNum) {
      latestNum = labelNum;
    }
  });
  return `Demo ${latestNum + 1}`;
}
function openCreateModal() {
  createChangeSetName.value = getGeneratedChangesetName();
  createModalRef.value?.open();
}

watch(
  // have to also watch for the modals existing since they may not exist immediately on mount
  [openChangeSets, createModalRef, selectModalRef],
  () => {
    if (!openChangeSets.value.length) {
      openCreateModal();
    } else if (!selectedChangeSetId.value) {
      selectModalRef.value?.open();
    }
  },
  { immediate: true },
);
</script>
