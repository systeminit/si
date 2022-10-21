<template>
  <div class="bg-neutral-800 rounded text-left flex flex-col">
    <div class="flex py-2 border-b border-black px-3 capitalize">
      {{ confirmation.title }}
    </div>

    <div class="w-full flex flex-col px-3 py-3 gap-2 text-sm">
      <StatusMessageBox :status="confirmation.status" type="confirmation"
        >Status: {{ confirmation.status }}
      </StatusMessageBox>

      <div v-if="confirmation.description">
        <b>Description: </b>
        <p>{{ confirmation.description }}</p>
      </div>

      <div class="text-right">
        <button class="underline text-action-400" @click="openModal">
          View Details
        </button>
      </div>
    </div>

    <Modal size="2xl" :open="modalOpen" @close="closeModal">
      <template #title>{{ confirmation.title }}</template>
      <template #content>
        <div class="my-2">
          <StatusMessageBox :status="confirmation.status" />
        </div>

        <div v-if="confirmation.description" class="my-2">
          <b>Description: </b>
          <p>{{ confirmation.description }}</p>
        </div>

        <div
          v-if="confirmation.output?.length"
          class="flex flex-col my-2 p-2 border border-warning-600 text-warning-500 rounded"
        >
          <b>Raw Output:</b>
          <p
            v-for="(output, index) in confirmation.output"
            :key="index"
            class="text-sm break-all"
          >
            {{ output }}
          </p>
        </div>
      </template>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { ref } from "vue";
import StatusMessageBox from "@/molecules/StatusMessageBox.vue";
import Modal from "@/ui-lib/Modal.vue";
import { Confirmation } from "@/store/resources.store";

defineProps<{
  confirmation: Confirmation;
}>();

const modalOpen = ref(false);

const openModal = () => {
  modalOpen.value = true;
};

const closeModal = () => {
  modalOpen.value = false;
};
</script>
