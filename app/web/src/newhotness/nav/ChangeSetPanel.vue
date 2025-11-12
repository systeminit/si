<template>
  <div class="flex flex-row gap-xs items-end flex-1 min-w-[156px] max-w-fit">
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
        :hoverBorder="false"
        :class="youHaveNewChangeSet && 'new-change-set-alert'"
        @select="onSelectChangeSet"
        @secondaryAction="openRenameModal"
        @click="clearAnimation"
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
        !changeSet ||
        (ctx.headChangeSetId.value &&
          changeSet.id === ctx.headChangeSetId.value) ||
        createApi.inFlight.value ||
        changeSet.status === ChangeSetStatus.NeedsApproval
      "
      icon="trash"
      tone="destructive"
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
          <NewButton
            :disabled="validationState.isError"
            :requestStatus="createApi.requestStatuses.value"
            icon="plus-circle"
            label="Create change set"
            tone="action"
            loadingText="Creating Change Set"
            submit
            class="w-full"
          />
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
import { computed, ref, watch } from "vue";
import * as _ from "lodash-es";
import { useRoute, useRouter } from "vue-router";
import {
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
import { ChangeSet, ChangeSetStatus } from "@/api/sdf/dal/change_set";
import { reset } from "@/newhotness/logic_composables/navigation_stack";
import ChangesetRenameModal from "@/components/ChangesetRenameModal.vue";
import AbandonChangeSetModal from "./AbandonChangeSetModal.vue";
import * as heimdall from "../../store/realtime/heimdall";
import { routes, useApi } from "../api_composables";
import { useChangeSets } from "../logic_composables/change_set";
import { useContext } from "../logic_composables/context";

const CHANGE_SET_NAME_REGEX = /^(?!head).*$/i;

const props = defineProps<{
  changeSetId: string;
  workspaceId: string;
}>();

const selectedChangeSetId = ref(props.changeSetId);
watch(
  () => props.changeSetId,
  () => {
    selectedChangeSetId.value = props.changeSetId;
  },
);

const ctx = useContext();

const { openChangeSets, changeSet } = useChangeSets(computed(() => ctx));

// when change set list updates with new data
// are any of the new CS from me (or my AI agent?)
const youHaveNewChangeSet = ref(false);
watch([openChangeSets, ctx.changeSetId], ([newCS, _], [oldCS, _c]) => {
  const myOld = new Set(
    oldCS.filter((c) => c.createdByUserId === ctx.user?.pk).map((c) => c.id),
  );
  const myNew = new Set(
    newCS.filter((c) => c.createdByUserId === ctx.user?.pk).map((c) => c.id),
  );
  const diff = [...myNew].filter((id) => !myOld.has(id));
  if (diff.length > 1) youHaveNewChangeSet.value = true;
  else if (diff.length === 0) youHaveNewChangeSet.value = false;
  else if (diff[0] && diff[0] !== ctx.changeSetId.value)
    youHaveNewChangeSet.value = true;
});

const clearAnimation = () => {
  youHaveNewChangeSet.value = false;
};

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

const waitForChangeSetExists = (changeSetId: string): Promise<void> => {
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

      if (await heimdall.changeSetExists(props.workspaceId, changeSetId)) {
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
      await waitForChangeSetExists(newVal);
    }

    const name = route.name;
    heimdall.showInterest(props.workspaceId, newVal);
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

  const call = createApi.endpoint<ChangeSet>(routes.CreateChangeSet);
  const { req } = await call.post({ name: createChangeSetName.value });

  if (createApi.ok(req)) {
    const newChangeSetId = req.data.id;
    onSelectChangeSet(newChangeSetId);

    createApi.navigateToNewChangeSet(
      {
        name: "new-hotness",
        params: {
          workspacePk: ctx.workspacePk.value,
          changeSetId: req.data.id,
        },
      },
      req.data.id,
    );

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

<style scoped>
/* 
  Taken right from the lobby with minimal alterations (jobelenus)
  Note*(victor): These styles exist to power the spinning border on the lobby terminal */

/*
  We need to declare --angle as a property with an initial value so that keyframes can correctly interpolate it
  If we don't, the keyframes below would only blip between the declared states
 */
@property --angle {
  syntax: "<angle>";
  inherits: false;
  initial-value: 0deg;
}
@keyframes borderRotate {
  100% {
    --angle: 360deg;
  }
}

@keyframes pulse {
  0%,
  35%,
  70%,
  100% {
    background-color: #000;
  }
  50% {
    background-color: rgba(134, 239, 172, 0.25); /* success-300 */
  }
}

.new-change-set-alert {
  border: 1px solid;
  /*
    use a spinning conic-gradient as the border image. It looks like this: https://www.geeksforgeeks.org/css/css-conic-gradient-function/
    But "masked" through the border
  */
  border-image: conic-gradient(
      from var(--angle),
      #333,
      #333 0.65turn,
      #86efac 1turn /* success-300 */
    )
    1;
  /*
    Enable the animation. Although the rotation is linear, since it's showing through a rectangular shape,
    both the moving speed and the trail length vary depending on the position. We could fudge the border speed by
    compensating on the keyframes vs the "radius" of each border position but this wouldn't fix the trail so
    we chose to use this as is.
  */
  animation: borderRotate 1500ms linear infinite forwards, pulse 5s infinite;
  /* border-radius does not interact with border images, so this all black mask that takes the size of the div makes it round again */
  mask-image: radial-gradient(#000 0, #000 0);
}
</style>
