<template>
  <div>
    <section class="px-[0.9375rem] my-[0.625rem]">
      <h1
        class="mb-[0.3125rem] text-neutral-900 type-bold-sm dark:text-neutral-50"
      >
        Change Set
      </h1>

      <div class="mb-3 flex items-center gap-x-[0.9375rem]">
        <SelectMenu
          v-model="changeSet"
          :options="openChangeSets"
          class="flex-grow"
          :disabled="showCreateDialog"
          @change="updateSelectedChangeSet"
        />
        <PrimaryActionButtonXSmall
          label="Apply"
          icon-style="left"
          icon="git-merge"
          @click="applyChangeSet"
        />
      </div>
    </section>
    <ChangeSetPanelDialog :show="showCreateDialog" @close="closeCreateDialog">
      <template #title>Create Change Set</template>
      <template #error>
        <div
          v-if="createChangeSetErrorMessage"
          class="mb-4 border bg-warning-300 py-1 px-2"
        >
          <TertiaryNeutralButtonXSmall
            label="Clear error message"
            icon-style="alone"
            icon="x"
            class="float-right"
            @click="createChangeSetErrorMessage = ''"
          />
          <p class="type-bold-xs">Failed to create Change Set</p>
          <p class="type-italic-xs">
            {{ createChangeSetErrorMessage }}
          </p>
        </div>
      </template>
      <template #body>
        <div>
          <p class="pb-2 type-regular-sm">
            Modeling a configuration or extending SI happens within
            <b>Change Sets</b>. Think of these like light-weight branches,
            allowing you to experiment freely without risk of impacting
            production systems.
          </p>
          <p class="type-regular-sm">
            Please give your <b>Change Set</b> a name below, and click the
            Create button.
          </p>
        </div>
        <div class="pt-2">
          <label for="changeSetName" class="type-medium-xs"
            >Change Set Name:</label
          >
          <input
            id="newChangeSetName"
            v-model="createChangeSetName"
            type="text"
            name="changeSetName"
            class="block w-full rounded-[0.1875rem] border-neutral-300 bg-shade-0 text-neutral-900 shadow-sm type-regular-xs hover:border-neutral-400 focus:border-neutral-500 focus:outline-none focus:ring-1 focus:ring-action-500 focus:ring-offset-2 dark:border-neutral-600 dark:bg-neutral-800 dark:text-neutral-50"
            placeholder="name"
            @keyup.enter="createChangeSet"
          />
          <p class="pt-4 type-regular-sm">
            In the future, you can create new change sets by selecting
            <b>- new -</b> in the <b>Change Set drop down</b>.
          </p>
        </div>
      </template>
      <template #buttons>
        <div class="flex flex-row-reverse justify-between pt-2">
          <PrimarySuccessButtonXSmall
            label="Create"
            icon-style="left"
            icon="plus-square"
            :disabled="createButtonDisabled"
            @click="createChangeSet"
          />
          <TertiaryDestructiveButtonXSmall
            label="Cancel"
            icon-style="left"
            icon="trash"
            @click="closeCreateDialog"
          />
        </div>
      </template>
    </ChangeSetPanelDialog>
    <ChangeSetPanelDialog :show="showSelectDialog" @close="closeSelectDialog">
      <template #title>Select Change Set</template>
      <template #body>
        <div class="type-regular-sm pb-2">
          <p>
            Select the Change Set you would like to resume working in, or select
            <b>- new -</b> to create a new Change Set.
          </p>
        </div>
        <div>
          <SelectMenu
            v-model="changeSet"
            :options="openChangeSets"
            class="flex-grow"
            @change="updateSelectedChangeSet"
          />
        </div>
      </template>
      <template #buttons>
        <div class="mt-2 flex flex-row justify-between">
          <TertiaryDestructiveButtonXSmall
            label="Cancel"
            icon-style="left"
            icon="trash"
            @click="closeSelectDialog"
          />
        </div>
      </template>
    </ChangeSetPanelDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { refFrom, untilUnmounted } from "vuse-rx";

import PrimaryActionButtonXSmall from "@/molecules/PrimaryActionButtonXSmall.vue";
import PrimarySuccessButtonXSmall from "@/molecules/PrimarySuccessButtonXSmall.vue";
import TertiaryDestructiveButtonXSmall from "@/molecules/TertiaryDestructiveButtonXSmall.vue";
import TertiaryNeutralButtonXSmall from "@/molecules/TertiaryNeutralButtonXSmall.vue";
import ChangeSetPanelDialog from "./ChangeSetPanelDialog.vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { Workspace } from "@/api/sdf/dal/workspace";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { WorkspaceService } from "@/service/workspace";

// The "create new change set" option
const CHANGE_SET_NEW = { label: "- new -", value: -3 };

// The default change sets list
const DEFAULT_CHANGE_SETS = [CHANGE_SET_NEW];

// The list of open change sets
const openChangeSets = ref<Option[]>(DEFAULT_CHANGE_SETS);

// The currently selected change set with the value being its primary key
const changeSet = ref(CHANGE_SET_NEW);

// Determines whether or not to display a dialog
const showDialog = ref<false | "create" | "select">(false);

// The name for a new change set
const createChangeSetName = ref("");

// An optional error message to be displayed when creating a change set fails
const createChangeSetErrorMessage = ref<string | null>(null);

// Determines if the create new change set button should be disabled
const createButtonDisabled = computed(() => {
  return createChangeSetName.value.length < 1;
});

// Determines whether or not to display create dialog
const showCreateDialog = computed(() => {
  return showDialog.value === "create";
});

// Determines whether or not to display select dialog
const showSelectDialog = computed(() => {
  return showDialog.value === "select";
});

// Current workspace
const currentWorkspace = refFrom<Workspace | null>(
  WorkspaceService.currentWorkspace(),
);

// Current change set
const currentChangeSet = refFrom<ChangeSet | null>(
  ChangeSetService.currentChangeSet(),
);

const router = useRouter();

// When the change set selection is changed, ignoring selections that don't
// update the currently selected
const updateSelectedChangeSet = () => {
  if (changeSet.value.value == CHANGE_SET_NEW.value) {
    showDialog.value = "create";
  } else if (currentChangeSet.value?.pk != changeSet.value.value) {
    GlobalErrorService.setIfError(
      ChangeSetService.updateSelectedChangeSet({
        nextChangeSetPk: changeSet.value.value,
      }),
    );
    showDialog.value = false;
  }
};

// When creatintg a new change set, uses the name from the create dialog
const createChangeSet = () => {
  ChangeSetService.createChangeSet({
    changeSetName: createChangeSetName.value,
  }).subscribe(async (response) => {
    if (response.error) {
      // Display the error message in the create dialog if creation failed
      createChangeSetErrorMessage.value = response.error.message;
    } else {
      createChangeSetName.value = "";
      showDialog.value = false;
    }
  });
};

// When the create dialog is closed without creating a new change set
const closeCreateDialog = async () => {
  // If the list of open change sets is 1 or less then there are no open change
  // sets so we'll redirect to the view page as the user chose not to create a
  // new change set and the compose view requires an open change set.
  if (openChangeSets.value.length <= 1) {
    await navigateToView();
  } else {
    // Clear the name in the form
    createChangeSetName.value = "";
    // Update the change set select to return to the current change set
    // value--it will currently be the "new" option at this moment.
    if (currentChangeSet.value) {
      changeSet.value = {
        label: currentChangeSet.value.name,
        value: currentChangeSet.value.pk,
      };
      showDialog.value = false;
    } else {
      showDialog.value = "select";
    }
  }
};

// When the select dialog is closed without selecting a new change set
const closeSelectDialog = async () => {
  await navigateToView();
};

// Saves the current edit session and then applies the current change set
const applyChangeSet = async () => {
  ChangeSetService.applyChangeSet().subscribe(async (response) => {
    if (response.error) {
      GlobalErrorService.set(response);
    } else {
      await navigateToView();
    }
  });
};

// Navigates to the workspace view page
const navigateToView = async () => {
  if (currentWorkspace.value) {
    await router.push({
      name: "workspace-view",
      path: "/new/w/:workspaceId/v",
      params: { workspaceId: currentWorkspace.value.id },
    });
  } else {
    // Fallback to the workspace list page in the case we can't yet determine
    // the current workspace (likely due to an observable race).
    await router.push({
      name: "workspace-multiple",
      path: "/new/w",
    });
  }
};

// Keeps the list of open change sets up to date
untilUnmounted(ChangeSetService.listOpenChangeSets()).subscribe((response) => {
  if (response.error) {
    GlobalErrorService.set(response);
    // If we encounter an error, continue with an empty option list
    openChangeSets.value = DEFAULT_CHANGE_SETS;
  } else {
    // If no open change sets are returned, display the create change set
    // dialog
    if (response.list.length == 0) {
      showDialog.value = "create";
    }
    openChangeSets.value = [...response.list, ...DEFAULT_CHANGE_SETS];
  }
});

// Updates the currently selected change set if it is updated externally
untilUnmounted(ChangeSetService.currentChangeSet()).subscribe(
  (currentChangeSet) => {
    if (currentChangeSet) {
      changeSet.value = {
        label: currentChangeSet.name,
        value: currentChangeSet.pk,
      };
    } else {
      showDialog.value = "select";
      changeSet.value = CHANGE_SET_NEW;
    }
  },
);
</script>
