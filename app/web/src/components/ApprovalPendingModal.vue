<template>
  <div>
    <!-- this modal is for the voting process -->
    <Modal ref="modalRef" title="Pending Approvals" size="lg">
      <div class="max-h-[70vh] overflow-hidden flex flex-col">
        <div class="text-md mb-xs">
          These change sets have been submitted for approval to be merged to
          HEAD. Select one to approve or reject it.
        </div>
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
// TODO(WENDY) - this mock modal still needs code for -
// Listing the pending approvals
// Selecting and displaying an individual approval with Reject/Approve buttons
import * as _ from "lodash-es";
import { Modal } from "@si/vue-lib/design-system";
import { ref } from "vue";

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

function openModalHandler() {
  modalRef.value?.open();
}

// function closeVotingModalHandler() {
//   modalRef.value?.close();
// }

defineExpose({ open: openModalHandler });
</script>
