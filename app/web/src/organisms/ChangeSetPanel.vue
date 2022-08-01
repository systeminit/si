<template>
  <div>
    <section class="px-[0.9375rem]">
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
          @change="changeSetChanged"
        />
        <PrimaryActionButtonXSmall
          label="Apply"
          icon-style="left"
          icon="git-merge"
        />
      </div>
      <div class="mb-3 flex items-center gap-x-[0.9375rem]">
        <TertiaryDestructiveButtonXSmall
          label="Discard"
          icon-style="left"
          icon="x"
          class="flex-grow"
        />
        <PrimarySuccessButtonXSmall
          label="Commit"
          icon-style="left"
          icon="git-branch"
          class="flex-grow"
        />
      </div>
    </section>
    <TransitionRoot :show="showCreateDialog" as="template">
      <Dialog as="div" class="relative z-50" @close="closeCreateDialog">
        <TransitionChild
          as="template"
          enter="ease-out duration-300"
          enter-from="opacity-0"
          enter-to="opacity-100"
          leave="ease-in duration-200"
          leave-from="opacity-100"
          leave-to="opacity-0"
        >
          <div
            class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
          />
        </TransitionChild>

        <div class="fixed inset-0 z-10 overflow-y-auto">
          <div
            class="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0"
          >
            <TransitionChild
              as="template"
              enter="ease-out duration-300"
              enter-from="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
              enter-to="opacity-100 translate-y-0 sm:scale-100"
              leave="ease-in duration-200"
              leave-from="opacity-100 translate-y-0 sm:scale-100"
              leave-to="opacity-0 translate-y-4 sm:translate-y-0 sm:scale-95"
            >
              <DialogPanel
                class="relative transform overflow-hidden rounded-lg bg-shade-0 text-neutral-900 px-4 pt-5 pb-4 text-left shadow-xl transition-all dark:bg-shade-100 dark:text-neutral-50 sm:my-8 sm:w-full sm:max-w-sm sm:p-6"
              >
                <div>
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
                  <div>
                    <DialogTitle as="h3" class="mb-2 type-bold-xs">
                      Create Change Set...
                    </DialogTitle>
                    <div class="mb-4">
                      <p class="type-italic-xs">
                        Lorem ipsum dolor sit amet consectetur adipisicing elit.
                        Consequatur amet labore.
                      </p>
                    </div>
                    <div>
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
                      <p class="my-2 type-italic-xs">
                        Lorem ipsum dolor sit amet consectetur adipisicing elit.
                        Consequatur amet labore.
                      </p>
                    </div>
                  </div>
                </div>
                <div class="flex flex-row-reverse justify-between">
                  <PrimarySuccessButtonXSmall
                    label="Create"
                    icon-style="left"
                    icon="plus"
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
              </DialogPanel>
            </TransitionChild>
          </div>
        </div>
      </Dialog>
    </TransitionRoot>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { refFrom, untilUnmounted } from "vuse-rx";
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from "@headlessui/vue";

import PrimaryActionButtonXSmall from "@/molecules/PrimaryActionButtonXSmall.vue";
import PrimarySuccessButtonXSmall from "@/molecules/PrimarySuccessButtonXSmall.vue";
import TertiaryDestructiveButtonXSmall from "@/molecules/TertiaryDestructiveButtonXSmall.vue";
import TertiaryNeutralButtonXSmall from "@/molecules/TertiaryNeutralButtonXSmall.vue";
import SelectMenu, { Option } from "@/molecules/SelectMenu.vue";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { Workspace } from "@/api/sdf/dal/workspace";
import { ChangeSetService } from "@/service/change_set";
import { GlobalErrorService } from "@/service/global_error";
import { WorkspaceService } from "@/service/workspace";

// The "create new change set" option
const CHANGE_SET_NEW = { label: ": new :", value: -3 };

// The default change sets list
const DEFAULT_CHANGE_SETS = [CHANGE_SET_NEW];

// The list of open change sets
const openChangeSets = ref<Option[]>(DEFAULT_CHANGE_SETS);

// The currently selected change set with the value being its primary key
const changeSet = ref(CHANGE_SET_NEW);

// Determines whether or not to display create dialog
const showCreateDialog = ref(false);

// The name for a new change set
const createChangeSetName = ref("");

// An optional error message to be displayed when creating a change set fails
const createChangeSetErrorMessage = ref<string | null>(null);

// Determines if the create new change set button should be disabled
const createButtonDisabled = computed(() => {
  return createChangeSetName.value.length < 1;
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

// When the change set selection is changed
const changeSetChanged = () => {
  if (changeSet.value.value == CHANGE_SET_NEW.value) {
    showCreateDialog.value = true;
  } else {
    GlobalErrorService.setIfError(
      ChangeSetService.getChangeSet({ pk: changeSet.value.value }),
    );
  }
};

// When creatintg a new change set, uses the name from the create dialog
const createChangeSet = async () => {
  ChangeSetService.createChangeSet({
    changeSetName: createChangeSetName.value,
  }).subscribe(async (response) => {
    if (response.error) {
      // Display the error message in the create dialog if creation failed
      createChangeSetErrorMessage.value = response.error.message;
    } else {
      createChangeSetName.value = "";
      showCreateDialog.value = false;
    }
  });
};

// When the create dialog is closed without creating a new change set
const closeCreateDialog = async () => {
  // If the list of open change sets is 1 or less then there are no open change
  // sets so we'll redirect to the view page as the user chose not to create a
  // new change set and the compose view requires an open change set.
  if (openChangeSets.value.length <= 1) {
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
    }
    showCreateDialog.value = false;
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
      showCreateDialog.value = true;
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
      changeSet.value = CHANGE_SET_NEW;
    }
  },
);
</script>
