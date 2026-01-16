<template>
  <!-- this modal is for the abandoning change sets -->
  <Modal ref="modalRef" title="Abandon Change Set?">
    <div class="text-md mb-xs">
      Are you sure that you want to abandon change set
      <span class="italic font-bold">
        {{ changeSet?.name }}
      </span>
      and return to HEAD?
    </div>
    <div class="text-sm mb-sm">Once abandoned, a change set cannot be recovered.</div>
    <div class="flex flex-row items-center w-full gap-sm">
      <VButton label="Cancel" variant="ghost" tone="warning" icon="x" @click="closeModalHandler" />
      <template v-if="!changeSetsStore.headSelected">
        <VButton
          label="Abandon Change Set"
          tone="destructive"
          class="flex-grow"
          icon="trash"
          loadingText="Abandoning Change Set"
          @click="abandonHandler"
        />
      </template>
    </div>
  </Modal>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { VButton, Modal } from "@si/vue-lib/design-system";
import { computed, ref } from "vue";

import { useChangeSetsStore } from "@/store/change_sets.store";

const changeSetsStore = useChangeSetsStore();

const modalRef = ref<InstanceType<typeof Modal> | null>(null);
const changeSet = computed(() => changeSetsStore.selectedChangeSet);

async function openModalHandler() {
  if (changeSet?.value?.name === "HEAD") return;
  modalRef.value?.open();
}

function closeModalHandler() {
  modalRef.value?.close();
}

function abandonHandler() {
  changeSetsStore.ABANDON_CHANGE_SET();
  closeModalHandler();
}

defineExpose({ open: openModalHandler });
</script>
