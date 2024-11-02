<template>
  <div>
    <!-- this modal is for the voting process -->
    <Modal ref="modalRef" title="Changes To Be Applied">
      <div class="max-h-[70vh] overflow-hidden flex flex-col">
        <div class="text-md mb-xs">
          Applying this change set may create, modify, or destroy real resources
          in the cloud.
        </div>
        <div class="text-sm mb-sm">
          These actions will be applied to the real world:
        </div>
        <div
          class="flex-grow overflow-y-auto mb-sm border border-neutral-100 dark:border-neutral-700"
        >
          <ActionsList slim kind="proposed" noInteraction />
        </div>
        <div
          class="flex flex-row w-full items-center justify-center gap-sm mt-xs"
        >
          <VButton
            label="Cancel"
            icon="x"
            variant="ghost"
            tone="warning"
            @click="closeModalHandler"
          />
          <VButton
            icon="tools"
            loadingText="Merging Changes"
            :label="userIsApprover ? 'Approve and Apply' : 'Request Approval'"
            class="grow"
            @click="applyButtonHandler"
          />
        </div>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { VButton, Modal } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useAuthStore } from "@/store/auth.store";
import ActionsList from "./Actions/ActionsList.vue";

const changeSetsStore = useChangeSetsStore();
const authStore = useAuthStore();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);
const changeSet = computed(() => changeSetsStore.selectedChangeSet);

const userIsApprover = ref(false);

async function openModalHandler() {
  if (changeSet?.value?.name === "HEAD") return;

  const data = await changeSetsStore.FETCH_CHANGE_SETS_V2();
  const approvers = data.rawResponseData?.approvers;
  const userPk = authStore.user?.pk;
  if (approvers && userPk && approvers.includes(userPk)) {
    userIsApprover.value = true;
  }

  modalRef.value?.open();
}

function closeModalHandler() {
  modalRef.value?.close();
}

function applyButtonHandler() {
  if (userIsApprover.value) {
    // TODO(Wendy) - not sure if this is right, need to figure out integration!
    if (authStore.user) {
      changeSetsStore.FORCE_APPLY_CHANGE_SET(authStore.user.name);
      closeModalHandler();
    }
  } else {
    // TODO(Wendy) - not sure if this is right, need to figure out integration!
    changeSetsStore.REQUEST_CHANGE_SET_APPROVAL();
    closeModalHandler();
  }
}

defineExpose({ open: openModalHandler });
</script>
