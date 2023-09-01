<template>
  <div class="border-b-2 dark:border-neutral-500 mb-2 flex-shrink-0">
    <section class="p-sm">
      <div class="flex items-center gap-x-xs">
        <VormInput
          class="flex-grow"
          type="dropdown"
          :modelValue="selectedChangeSetId"
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
          <VButton
            ref="mergeButtonRef"
            icon="git-merge"
            size="md"
            loadingText="Merging"
            label="Merge"
            :requestStatus="applyChangeSetReqStatus"
            :disabled="statusStoreUpdating"
            @click="applyChangeSet"
          />
        </VormInput>
      </div>
    </section>

    <Modal
      ref="createModalRef"
      title="Create Change Set"
      @close="onCreateModalClose"
    >
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

    <Modal ref="selectModalRef" noExit title="Select Change Set">
      <Stack>
        <p>
          Select the Change Set you would like to resume working in, or select
          <b>- new -</b> to create a new Change Set.
        </p>
        <VormInput
          type="dropdown"
          noLabel
          :modelValue="selectedChangeSetId"
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
        <div
          v-else-if="changeSetMergeStatus.isSuccess"
          class="gap-2 items-center flex flex-col"
        >
          <span class="text-3xl">
            {{ celebrate }} Change Set Merged! {{ celebrate }}
          </span>
        </div>
      </template>
    </Wipe>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import JSConfetti from "js-confetti";
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
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useStatusStore } from "@/store/status.store";
import Wipe from "./Wipe.vue";

const wipeRef = ref<InstanceType<typeof Wipe>>();
const mergeButtonRef = ref();

const workspacesStore = useWorkspacesStore();
const selectedWorkspacePk = computed(() => workspacesStore.selectedWorkspacePk);

const changeSetsStore = useChangeSetsStore();
const openChangeSets = computed(() => changeSetsStore.openChangeSets ?? []);
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

const applyChangeSetReqStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

const celebrationEmoji = [
  "🎉",
  "🎊",
  "✨",
  "🔥",
  "⚡️",
  "🥳",
  "🍻",
  "🍺",
  "🥂",
  "🍾",
];

const celebrate = ref("🎉");
let jsConfetti: JSConfetti;
const confettis = [
  { emojis: ["🎉"] },
  { emojis: ["🍿"] },
  { emojis: ["🤘", "🤘🏻", "🤘🏼", "🤘🏽", "🤘🏾", "🤘🏿"] },
  { emojis: ["❤️", "🧡", "💛", "💚", "💙", "💜"] },
  { emojis: ["🍾", "🍷", "🍸", "🍹", "🍺", "🥂", "🍻"] },
  { emojis: ["🏳️‍🌈", "🏳️‍⚧️", "⚡️", "🌈", "✨", "🔥"] },
];
onMounted(() => {
  jsConfetti = new JSConfetti({
    canvas:
      (document.getElementById("confetti") as HTMLCanvasElement) || undefined,
  });
});

const changeSetMergeStatus =
  changeSetsStore.getRequestStatus("APPLY_CHANGE_SET");

// Applies the current change set
const applyChangeSet = async () => {
  if (!wipeRef.value) return; // bail if the wipe doesn't exist

  // Pick a celebration emoji!
  celebrate.value = _.sample(celebrationEmoji)!; // eslint-disable-line @typescript-eslint/no-non-null-assertion

  // Run both the wipe and the change set apply in parallel
  const wipeDone = wipeRef.value.open(mergeButtonRef.value.$el);

  await changeSetsStore.APPLY_CHANGE_SET();
  await wipeDone;

  // when the change set is done done, check if the change set apply was successful
  if (changeSetMergeStatus.value.isSuccess) {
    await jsConfetti.addConfetti(_.sample(confettis));
    wipeRef.value?.close();
    await navigateToFixMode();
  }
};

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
    if (!openChangeSets.value?.length) {
      openCreateModal();
    } else if (!selectedChangeSetId.value) {
      selectModalRef.value?.open();
    }
  },
  { immediate: true },
);

// Navigates to the workspace fix page
const navigateToFixMode = async () => {
  if (selectedWorkspacePk.value) {
    await router.push({
      name: " workspace-compose",
      params: { workspacePk: selectedWorkspacePk.value },
    });
  } else {
    // Fallback to the workspace list page in the case we can't yet determine
    // the current workspace (likely due to an observable race).
    await router.push({ name: "workspace-index" });
  }
};

const statusStore = useStatusStore();
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});

function onCreateModalClose() {
  if (!selectedChangeSetId.value) navigateToFixMode();
}
</script>
