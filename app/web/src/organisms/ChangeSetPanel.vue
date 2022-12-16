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
          <VButton2
            ref="mergeButtonRef"
            icon="git-merge"
            size="md"
            loading-text="Merging"
            label="Merge"
            :request-status="applyChangeSetReqStatus"
            hover-glow
            :disabled="statusStoreUpdating"
            @click="applyChangeSet"
          />
        </VormInput>
      </div>
    </section>

    <Modal
      :open="showDialog === 'create'"
      size="sm"
      hide-top-close-button
      disable-close
      type="custom"
      @close="onCloseCreateDialog"
    >
      <template #title>Create Change Set</template>
      <template #content>
        <form @submit.prevent="onCreateChangeSet">
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
          <div class="flex flex-row-reverse gap-sm py-3">
            <VButton2
              :disabled="validationState.isError"
              tone="success"
              icon="plus-circle"
              label="Create"
              loading-text="Creating Change Set"
              :request-status="createChangeSetReqStatus"
              class="flex-grow"
              submit
            />
            <VButton2
              v-if="changeSetsStore.openChangeSets.length > 0"
              tone="destructive"
              variant="ghost"
              icon="x-circle"
              label="Cancel"
              @click="onCloseCreateDialog"
            />
          </div>
        </form>
      </template>
    </Modal>

    <Modal
      :open="showDialog === 'select'"
      size="sm"
      hide-top-close-button
      disable-close
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
          <VButton2 icon="plus-circle" @click="switchToCreateMode">
            Create a new change set
          </VButton2>
        </Stack>
      </template>
    </Modal>

    <Wipe ref="wipeRef">
      <template #duringWipe>
        <VButton2
          icon="git-merge"
          size="md"
          loading-text="Merging"
          label="Merge"
          loading
        />
      </template>
      <template #afterWipe>
        <div
          v-if="changeSetMergeStatus.isPending"
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
          <span class="text-md italic pt-sm">
            Preparing your recommendations...
          </span>
        </div>
      </template>
    </Wipe>
  </div>
</template>

<script lang="ts" setup>
import { computed, onMounted, ref, watch } from "vue";
import _ from "lodash";
import { useRoute, useRouter } from "vue-router";
import JSConfetti from "js-confetti";
import { useChangeSetsStore } from "@/store/change_sets.store";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import VormInputOption from "@/ui-lib/forms/VormInputOption.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import Divider from "@/ui-lib/layout/Divider.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import Modal from "@/ui-lib/Modal.vue";
import { useValidatedInputGroup } from "@/ui-lib/forms/helpers/form-validation";
import Icon from "@/ui-lib/icons/Icon.vue";
import { useStatusStore } from "@/store/status.store";
import Wipe from "@/ui-lib/Wipe.vue";

const wipeRef = ref<InstanceType<typeof Wipe>>();
const mergeButtonRef = ref();

const workspacesStore = useWorkspacesStore();
const selectedWorkspaceId = computed(() => workspacesStore.selectedWorkspaceId);

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

// Determines whether or not to display a dialog
const showDialog = ref<false | "create" | "select">(false);

// The name for a new change set
const createChangeSetName = ref("");

const { validationState, validationMethods } = useValidatedInputGroup();

function onSelectChangeSet(newVal: string | "NEW") {
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
  {},
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

// Saves the current edit session and then applies the current change set
const applyChangeSet = async () => {
  if (!wipeRef.value) return; // bail if the wipe doesn't exist

  // Pick a celebration emoji!
  celebrate.value = _.sample(celebrationEmoji) || "🎉";

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

const switchToCreateMode = () => {
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

const statusStore = useStatusStore();
const statusStoreUpdating = computed(() => {
  if (statusStore.globalStatus) {
    return statusStore.globalStatus.isUpdating;
  } else return false;
});

function onCloseCreateDialog() {
  showDialog.value = false;
  if (!selectedChangeSetId.value) navigateToFixMode();
}

function onCloseSelectDialog() {
  showDialog.value = false;
  if (!selectedChangeSetId.value) navigateToFixMode();
}

defineExpose({
  showDialog,
});
</script>
