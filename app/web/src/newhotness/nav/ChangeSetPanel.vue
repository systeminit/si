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

    <VButton
      v-tooltip="{
        content: 'Create Change Set',
      }"
      data-testid="create-change-set-button"
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
      data-testid="abandon-change-set-button"
      :disabled="
        !changeSet ||
        (ctx.headChangeSetId.value &&
          changeSet.id === ctx.headChangeSetId.value) ||
        createApi.inFlight.value ||
        changeSet.status === ChangeSetStatus.NeedsApproval
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
              :requestStatus="createApi.requestStatuses.value"
              :class="
                clsx(
                  'flex-grow !text-sm !border !cursor-pointer !px-xs',
                  themeClasses(
                    '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                    '!text-neutral-100 !bg-[#1264BF] !border-[#318AED] hover:!bg-[#2583EC]',
                  ),
                )
              "
              icon="plus-circle"
              label="Create change set"
              loadingText="Creating Change Set"
              submit
            />
          </div>
        </Stack>
      </form>
    </Modal>
    <AbandonChangeSetModal
      v-if="changeSet"
      ref="abandonModalRef"
      :changeSet="changeSet"
    />
    <ChangesetRenameModal ref="renameModalRef" />
  </div>
</template>

<script lang="ts" setup>
import { computed, inject, ref, watch } from "vue";
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
  themeClasses,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { tw } from "@si/vue-lib";
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { reset } from "@/newhotness/logic_composables/navigation_stack";
import ChangesetRenameModal from "@/components/ChangesetRenameModal.vue";
import AbandonChangeSetModal from "./AbandonChangeSetModal.vue";
import * as heimdall from "../../store/realtime/heimdall";
import { routes, useApi } from "../api_composables";
import { useChangeSets } from "../logic_composables/change_set";
import { assertIsDefined, Context } from "../types";

const CHANGE_SET_NAME_REGEX = /^(?!head).*$/i;

const props = defineProps<{
  changeSetId: string;
}>();

const selectedChangeSetId = ref(props.changeSetId);
watch(
  () => props.changeSetId,
  () => {
    selectedChangeSetId.value = props.changeSetId;
  },
);

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);
const { openChangeSets, changeSet } = useChangeSets(computed(() => ctx));

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

const calculateShowSecondaryAction = (option: { label: string }) => {
  return option.label.toUpperCase() !== "HEAD";
};

const abandonModalRef = ref<InstanceType<typeof AbandonChangeSetModal> | null>(
  null,
);
const openAbandonConfirmationModal = () => {
  abandonModalRef.value?.open();
};

const renameModalRef = ref<InstanceType<typeof ChangesetRenameModal> | null>(
  null,
);
function openRenameModal(option: { value: string; label: string }) {
  renameModalRef.value?.open(option.value, option.label);
}

const createModalRef = ref<InstanceType<typeof Modal>>();

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
    openCreateModal();
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

const createApi = useApi();
async function onCreateChangeSet() {
  if (validationMethods.hasError()) return;

  const call = createApi.endpoint<{ changeSet: ChangeSet }>(
    routes.CreateChangeSet,
  );
  const { req } = await call.post({ changeSetName: createChangeSetName.value });

  if (createApi.ok(req)) {
    const newChangeSetId = req.data.changeSet.id;
    onSelectChangeSet(newChangeSetId);
    createModalRef.value?.close();
  }
}

function openCreateModal() {
  if (createModalRef.value?.isOpen) return;
  createChangeSetName.value = "";
  createModalRef.value?.open();
}

defineExpose({ openCreateModal });
</script>
