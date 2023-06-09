<template>
  <div class="flex-shrink-0">
    <div class="flex flex-col gap-1">
      <div class="text-xs font-medium capsize">CHANGE SET:</div>

      <div class="flex-grow flex gap-2.5">
        <VormInput
          class="flex-grow font-bold"
          size="sm"
          type="dropdown"
          no-label
          :model-value="selectedChangeSetId"
          :options="changeSetDropdownOptions"
          @update:model-value="onSelectChangeSet"
        />

        <VButton
          tone="action"
          variant="ghost"
          icon="git-branch"
          size="sm"
          @click="openCreateModal"
        />
      </div>
    </div>

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
  Stack,
  Modal,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { useChangeSetsStore, ChangeSetId } from "@/store/change_sets.store";
import Wipe from "./Wipe.vue";

const wipeRef = ref<InstanceType<typeof Wipe>>();

const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const changeSetDropdownOptions = computed(() =>
  _.map(openChangeSets.value ?? [], (cs) => ({ value: cs.id, label: cs.name })),
);

const router = useRouter();
const route = useRoute();

const createModalRef = ref<InstanceType<typeof Modal>>();

// The name for a new change set
const createChangeSetName = ref("");

const { validationState, validationMethods } = useValidatedInputGroup();

function onSelectChangeSet(newVal: string) {
  if (newVal && route.name) {
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
    createModalRef.value?.close();
  }
}

const createChangeSetReqStatus =
  changeSetsStore.getRequestStatus("CREATE_CHANGE_SET");

const changeSetMergeStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

function openCreateModal() {
  createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();
  createModalRef.value?.open();
}

watch(
  // have to also watch for the modals existing since they may not exist immediately on mount
  [openChangeSets, createModalRef, route, selectedChangeSetId],
  () => {
    if (!openChangeSets.value) return;

    if (!selectedChangeSetId.value) {
      tryAutoSelect();
    }
  },
  { immediate: true },
);

function routeToChangeSet(id: ChangeSetId, replace = false) {
  // reroutes to a specific changeset but keeps the same route name
  // so we can go from /auto/some-specific-page -> 1/some-specific-page
  router[replace ? "replace" : "push"]({
    name: route.name!, // eslint-disable-line @typescript-eslint/no-non-null-assertion
    params: {
      ...route.params,
      changeSetId: id,
    },
  });
}

// gets called on url change when id is "auto", and also when change sets are loaded
function tryAutoSelect() {
  const autoSelectChangeSetId = changeSetsStore.getAutoSelectedChangeSetId();
  if (autoSelectChangeSetId) {
    routeToChangeSet(autoSelectChangeSetId, true);
  } else {
    createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();
    onCreateChangeSet();
  }
}
</script>
