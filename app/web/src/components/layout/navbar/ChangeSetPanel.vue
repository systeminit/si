<template>
  <div>
    <div class="flex flex-col gap-1">
      <div class="text-xs font-medium capsize">CHANGE SET:</div>

      <div class="flex-grow flex gap-2.5">
        <VormInput
          ref="dropdownRef"
          class="flex-grow font-bold"
          size="sm"
          type="dropdown"
          noLabel
          :modelValue="selectedChangeSetId"
          :options="changeSetDropdownOptions"
          @update:model-value="onSelectChangeSet"
        />

        <VButton
          tone="action"
          variant="ghost"
          icon="git-branch-plus"
          size="sm"
          :disabled="fixesStore.fixesAreInProgress"
          @click="openCreateModal"
        />

        <VButton
          v-if="featureFlagStore.ABANDON_CHANGESET"
          tone="action"
          variant="ghost"
          icon="trash"
          size="sm"
          :disabled="fixesStore.fixesAreInProgress || !selectedChangeSetName"
          @click="abandonConfirmationModalRef.open()"
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
            requiredMessage="Please choose a name for your change set!"
          />
          <div class="flex flex-row-reverse gap-sm">
            <VButton
              :disabled="validationState.isError"
              tone="success"
              icon="plus-circle"
              label="Create change set"
              loadingText="Creating Change Set"
              :requestStatus="createChangeSetReqStatus"
              class="flex-grow"
              submit
            />
          </div>
        </Stack>
      </form>
    </Modal>

    <Modal ref="abandonConfirmationModalRef" title="Abandon Change Set?">
      <div class="flex flex-col gap-sm">
        <div class="text-md">
          Are you sure that you want to abandon change set
          <span class="italic font-bold">"{{ selectedChangeSetName }}"</span>
          and return to HEAD?
        </div>
        <div class="text-md">
          Once abandoned, a change set cannot be recovered.
        </div>
        <div class="flex flex-row items-center gap-sm">
          <VButton
            class="flex-grow"
            label="Confirm"
            tone="action"
            icon="trash"
            :requestStatus="abandonChangeSetReqStatus"
            loadingText="Abandoning Changeset..."
            @click="abandonChangesetHandler"
          />
          <VButton
            class="flex-grow"
            label="Cancel"
            tone="destructive"
            icon="x"
            @click="abandonConfirmationModalRef.close"
          />
        </div>
      </div>
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
import { onMounted, computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
  VButton,
  Icon,
  VormInput,
  Stack,
  Modal,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFixesStore } from "@/store/fixes.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import Wipe from "../../Wipe.vue";

const dropdownRef = ref();
const abandonConfirmationModalRef = ref();
const wipeRef = ref<InstanceType<typeof Wipe>>();

const changeSetsStore = useChangeSetsStore();
const featureFlagStore = useFeatureFlagsStore();
const fixesStore = useFixesStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const selectedChangeSetId = computed(() => changeSetsStore.selectedChangeSetId);
const selectedChangeSetName = computed(
  () => changeSetsStore.selectedChangeSet?.name,
);

const changeSetDropdownOptions = computed(() => {
  const cs: { value: string; label: string }[] = _.map(
    openChangeSets.value ?? [],
    (cs) => ({ value: cs.id, label: cs.name }),
  );
  cs.unshift({ value: nilId(), label: "head" });
  return cs;
});

const router = useRouter();
const route = useRoute();

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
onMounted(checkFirstLoad);

// The name for a new change set
const createChangeSetName = ref(changeSetsStore.getGeneratedChangesetName());

const { validationState, validationMethods } = useValidatedInputGroup();

function onSelectChangeSet(newVal: string) {
  if (newVal && route.name) {
    if (newVal === nilId()) newVal = "head";
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
  createChangeSetName.value = changeSetsStore.getGeneratedChangesetName();

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

const abandonChangeSetReqStatus =
  changeSetsStore.getRequestStatus("ABANDON_CHANGE_SET");

async function abandonChangesetHandler() {
  await changeSetsStore.ABANDON_CHANGE_SET();

  abandonConfirmationModalRef.value.close();

  if (route.name) {
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }

  await changeSetsStore.FETCH_CHANGE_SETS();

  // TODO(Wendy) - a temporary fix until we figure out and fix the bug where components from the abandoned changeset do not disappear from the diagram
  // eslint-disable-next-line no-restricted-globals
  location.reload();
}
</script>
