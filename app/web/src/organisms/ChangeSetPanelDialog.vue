<template>
  <TransitionRoot as="template">
    <Dialog as="div" class="relative z-50" @close="emit('close')">
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
              class="relative transform overflow-hidden rounded-md bg-shade-0 text-neutral-900 text-left shadow-xl transition-all dark:bg-neutral-800 dark:text-neutral-50 sm:w-full sm:max-w-sm"
            >
              <DialogTitle
                as="h3"
                class="type-regular-lg border-neutral-200 dark:border-neutral-900 border-2 pt-2 pb-2 px-3"
              >
                <slot name="title"></slot>
              </DialogTitle>
              <div class="type-regular-sm px-3">
                <slot name="error"></slot>
              </div>
              <div>
                <div class="px-3 pt-2">
                  <slot name="body"></slot>
                </div>
              </div>
              <div class="pl-4 pr-4 pb-3">
                <slot name="buttons"></slot>
              </div>
            </DialogPanel>
          </TransitionChild>
        </div>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script setup lang="ts">
import {
  Dialog,
  DialogPanel,
  DialogTitle,
  TransitionChild,
  TransitionRoot,
} from "@headlessui/vue";

const emit = defineEmits(["close"]);
</script>
