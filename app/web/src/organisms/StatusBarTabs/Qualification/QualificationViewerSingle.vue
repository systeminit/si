<template>
  <div class="bg-neutral-800 rounded text-left flex flex-col">
    <div class="flex py-2 border-b border-black px-3 capitalize">
      {{ qualification.title }}
    </div>

    <div class="w-full flex flex-col px-3 py-3 gap-2 text-sm">
      <StatusMessageBox :status="qualificationStatus">
        <template v-if="qualificationStatus === 'failure'">
          Something went wrong! Click "View Details" to see the output.
        </template>
        <template v-else-if="qualificationStatus === 'success'">
          Passed!
        </template>
        <template v-else> Qualification running, standby...</template>
      </StatusMessageBox>

      <div v-if="qualification.description">
        <b>Description: </b>
        <p>{{ qualification.description }}</p>
      </div>

      <div class="text-right">
        <button class="underline text-action-400" @click="openModal">
          View Details
        </button>
      </div>
    </div>

    <TransitionRoot :show="modalOpen" appear as="template">
      <Dialog as="div" class="relative z-50" @close="closeModal">
        <TransitionChild
          as="template"
          enter="duration-300 ease-out"
          enter-from="opacity-0"
          enter-to="opacity-100"
          leave="duration-200 ease-in"
          leave-from="opacity-100"
          leave-to="opacity-0"
        >
          <div class="fixed inset-0 bg-black bg-opacity-50" />
        </TransitionChild>

        <div class="fixed inset-0 overflow-y-auto">
          <div class="flex min-h-full items-center justify-center text-center">
            <TransitionChild
              as="template"
              enter="duration-300 ease-out"
              enter-from="opacity-0 scale-95"
              enter-to="opacity-100 scale-100"
              leave="duration-200 ease-in"
              leave-from="opacity-100 scale-100"
              leave-to="opacity-0 scale-95"
            >
              <DialogPanel
                class="w-full max-w-2xl transform overflow-hidden rounded bg-white dark:bg-neutral-900 text-left align-middle shadow-xl transition-all text-black dark:text-white"
              >
                <div
                  class="flex justify-between items-center py-2 border-b border-black px-2"
                >
                  <DialogTitle as="p" class="capitalize">
                    {{ qualification.title }}
                  </DialogTitle>
                  <VButton
                    hide-label
                    button-rank="tertiary"
                    button-type="neutral"
                    icon="x"
                    label="Close Dialog"
                    @click="closeModal"
                  />
                </div>

                <div class="w-full flex flex-col px-2 py-3 gap-3 text-sm">
                  <StatusMessageBox :status="qualificationStatus">
                    <template v-if="qualificationStatus === 'failure'">
                      Something went wrong!
                    </template>
                    <template v-else-if="qualificationStatus === 'success'">
                      Passed!
                    </template>
                    <template v-else>
                      Qualification running, standby...
                    </template>
                  </StatusMessageBox>

                  <div v-if="qualification.description">
                    <b>Description: </b>
                    <p>{{ qualification.description }}</p>
                  </div>

                  <div
                    v-if="qualification.output?.length"
                    class="flex flex-col p-2 border border-warning-600 text-warning-500 rounded"
                  >
                    <b>Raw Output:</b>
                    <p
                      v-for="(output, index) in qualification.output"
                      :key="index"
                      class="text-sm"
                    >
                      {{ output.line }}
                    </p>
                  </div>
                </div>
                <div class="py-1 px-2 border-t dark:border-black text-right">
                  <VButton
                    button-rank="tertiary"
                    button-type="neutral"
                    icon="x"
                    label="Close"
                    @click="closeModal"
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

<script lang="ts" setup>
import { computed, ref } from "vue";
import _ from "lodash";
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from "@headlessui/vue";
import { Qualification } from "@/api/sdf/dal/qualification";
import VButton from "@/molecules/VButton.vue";
import StatusMessageBox from "@/molecules/StatusMessageBox.vue";

const props = defineProps<{
  qualification: Qualification;
}>();

const qualificationStatus = computed(() => {
  if (_.isNil(props.qualification.result)) return "loading";

  if (props.qualification.result.success) return "success";

  return "failure";
});

const modalOpen = ref(false);

const openModal = () => {
  modalOpen.value = true;
};

const closeModal = () => {
  modalOpen.value = false;
};
</script>
