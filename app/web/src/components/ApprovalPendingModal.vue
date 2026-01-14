<template>
  <div>
    <!-- this modal is for the voting process -->
    <Modal ref="modalRef" title="Pending Approvals" size="lg">
      <div class="max-h-[70vh] overflow-hidden flex flex-col gap-xs">
        <div class="text-md pb-xs">
          These change sets have been submitted for approval to be merged to HEAD. Select one to approve or reject it.
        </div>
        <ApprovalPendingModalCard
          v-for="changeSet in pendingChangeSets"
          :key="changeSet.id"
          :changeSet="changeSet"
          @closeModal="closeModalHandler"
        />
      </div>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { Modal } from "@si/vue-lib/design-system";
import { ref, computed } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ApprovalPendingModalCard from "./ApprovalPendingModalCard.vue";

const changeSetsStore = useChangeSetsStore();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);

const pendingChangeSets = computed(() => changeSetsStore.changeSetsNeedingApproval);

function openModalHandler() {
  modalRef.value?.open();
}

function closeModalHandler() {
  modalRef.value?.close();
}

defineExpose({ open: openModalHandler });
</script>
