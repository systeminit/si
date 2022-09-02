<template>
  <TransitionRoot :show="open" appear as="template">
    <Dialog as="div" class="relative z-50" @close="emit('close')">
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
              class="w-full max-w-md transform overflow-hidden rounded bg-white dark:bg-neutral-900 text-left align-middle shadow-xl transition-all text-black dark:text-white"
            >
              <div
                class="flex justify-between items-center py-2 border-b border-black px-2"
              >
                <DialogTitle as="p" class="capitalize">
                  Controls For Diagram Editor
                </DialogTitle>
                <VButton
                  hide-label
                  button-rank="tertiary"
                  button-type="neutral"
                  icon="x"
                  label="Close Dialog"
                  @click="emit('close')"
                />
              </div>

              <div
                class="py-1 px-2 border-t dark:border-black flex flex-col place-content-center"
              >
                <div class="p-4 flex flex-row place-content-center">
                  <ul class="space-y-2">
                    <li>Mouse - Add, Create, and Select Components</li>
                    <li>Arrow Keys - Nudge Selected Component</li>
                    <li>Spacebar - Hold To Drag Scroll</li>
                    <li>Esc - Deselect Component</li>
                    <li>Backspace/Delete - Delete Selected Component</li>
                  </ul>
                </div>
                <VButton
                  button-rank="tertiary"
                  button-type="neutral"
                  icon="x"
                  label="Close"
                  @click="emit('close')"
                />
              </div>
            </DialogPanel>
          </TransitionChild>
        </div>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script lang="ts" setup>
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from "@headlessui/vue";
import VButton from "@/molecules/VButton.vue";

defineProps({
  open: { type: Boolean, default: false },
});

const emit = defineEmits<{
  (e: "close"): void;
}>();
</script>
