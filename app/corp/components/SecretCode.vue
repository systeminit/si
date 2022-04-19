<template>
  <ClientOnly>
    <TransitionRoot as="template" :show="modelValue">
      <Dialog
        as="div"
        class="fixed z-10 inset-0 overflow-y-auto"
        @close="closeDialog"
      >
        <div
          class="flex items-end justify-center min-h-screen pt-4 px-4 pb-20 text-center sm:block sm:p-0"
        >
          <TransitionChild
            as="template"
            enter="ease-out duration-300"
            enter-from="opacity-0"
            enter-to="opacity-100"
            leave="ease-in duration-200"
            leave-from="opacity-100"
            leave-to="opacity-0"
          >
            <DialogOverlay
              class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity"
            />
          </TransitionChild>

          <!-- This element is to trick the browser into centering the modal contents. -->
          <span
            class="hidden sm:inline-block sm:align-middle sm:h-screen"
            aria-hidden="true"
            >&#8203;</span
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
            <div
              class="relative inline-block align-bottom bg-slate-800 rounded-sm px-4 pt-5 pb-4 text-left overflow-hidden shadow-xl transform transition-all sm:my-8 sm:align-middle sm:max-w-lg sm:w-full sm:p-6"
            >
              <div>
                <div
                  class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-800"
                >
                  <EyeIcon class="h-6 w-6 text-white" aria-hidden="true" />
                </div>
                <div
                  v-if="!secretStore.secretAgent"
                  class="mt-3 text-center sm:mt-5"
                >
                  <DialogTitle
                    as="h3"
                    class="text-lg leading-6 font-bold text-gray-100"
                  >
                    Top Secret
                  </DialogTitle>
                  <div class="mt-2">
                    <p class="text-sm text-gray-500">
                      Enter the passphrase, Agent.
                    </p>
                    <div class="mt-2">
                      <label for="passphrase" class="sr-only">Passphrase</label>
                      <input
                        id="passphrase"
                        v-model="passphrase"
                        type="password"
                        name="passphrase"
                        class="shadow-sm focus:ring-red-500 focus:border-red-500 block w-full sm:text-sm border-gray-300 rounded-sm bg-slate-500 text-slate-200"
                        placeholder="you@example.com"
                      />
                    </div>
                  </div>
                </div>
              </div>
              <div
                class="mt-5 sm:mt-6 sm:grid sm:grid-cols-2 sm:gap-3 sm:grid-flow-row-dense"
              >
                <button
                  v-if="secretStore.secretAgent"
                  type="button"
                  class="w-full inline-flex justify-center rounded-sm border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:col-start-2 sm:text-sm"
                  @click="leaveInitiative"
                >
                  Leave the Initiative
                </button>
                <button
                  v-else
                  type="button"
                  class="w-full inline-flex justify-center rounded-sm border border-transparent shadow-sm px-4 py-2 bg-red-600 text-base font-medium text-white hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 sm:col-start-2 sm:text-sm"
                  @click="authenticate"
                >
                  Authenticate
                </button>

                <button
                  ref="cancelButtonRef"
                  type="button"
                  class="mt-3 w-full inline-flex justify-center rounded-sm border border-gray-300 shadow-sm px-4 py-2 bg-white text-base font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 sm:mt-0 sm:col-start-1 sm:text-sm"
                  @click="closeDialog"
                >
                  Cancel
                </button>
              </div>
            </div>
          </TransitionChild>
        </div>
      </Dialog>
    </TransitionRoot>
  </ClientOnly>
</template>

<script setup lang="ts">
import { defineProps, ref } from "vue";
import {
  Dialog,
  DialogOverlay,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from "@headlessui/vue";
import { EyeIcon } from "@heroicons/vue/outline";

const secretStore = useSecretStore();

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  // eslint-disable-next-line no-unused-vars
  (e: "update:modelValue", value: boolean): void;
}>();

const passphrase = ref("");

const closeDialog = () => {
  emit("update:modelValue", false);
};

const authenticate = () => {
  closeDialog();
  secretStore.authenticateSecretAgent(passphrase.value);
  passphrase.value = "";
};

const leaveInitiative = () => {
  closeDialog();
  passphrase.value = "";
  secretStore.leaveInitiative();
};
</script>
