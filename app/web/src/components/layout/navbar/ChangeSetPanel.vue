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
    <Modal ref="changeSetAppliedRef" noExit>
      <div
        class="bg-white dark:bg-neutral-700 rounded-lg flex flex-col items-center w-96 max-h-[90vh] shadow-md overflow-hidden pb-xs"
      >
        <div class="px-sm pt-sm pb-xs w-full">
          The change set you were in was merged by:
        </div>

        <div v-if="applyUser" class="pr-xs">
          <UserCard :user="applyUser" hideChangesetInfo />
        </div>
        <div class="px-sm pb-sm pt-xs w-full">
          You are now on Head. You can continue your work by creating a new
          change set or joining another existing change set.
        </div>
        <VButton
          label="Ok"
          variant="ghost"
          size="sm"
          @click="changeSetAppliedHandler()"
        />
      </div>
    </Modal>
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
import { storeToRefs } from "pinia";
import { nilId } from "@/utils/nilId";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFixesStore } from "@/store/fixes.store";
import UserCard from "@/components/layout/navbar/UserCard.vue";
import { usePresenceStore } from "@/store/presence.store";
import { useAuthStore } from "@/store/auth.store";
import Wipe from "../../Wipe.vue";

const dropdownRef = ref();
const changeSetAppliedRef = ref();
const wipeRef = ref<InstanceType<typeof Wipe>>();

const changeSetsStore = useChangeSetsStore();
const presenceStore = usePresenceStore();
const authStore = useAuthStore();
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

const { postApplyActor } = storeToRefs(changeSetsStore);
watch(postApplyActor, () => {
  if (
    postApplyActor.value !== null &&
    postApplyActor.value !== authStore.user?.pk
  ) {
    changeSetAppliedRef.value?.open();
  }
});

function changeSetAppliedHandler() {
  changeSetAppliedRef.value?.close();
  // Redirect the user to head changeset
  if (route.name) {
    router.push({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }
}

const applyUser = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  return presenceStore.usersById[changeSetsStore.postApplyActor!];
});
</script>
