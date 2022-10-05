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
            <DialogPanel :class="dialogPanelClasses">
              <div
                class="flex justify-between items-center py-2 border-b border-black px-2"
              >
                <DialogTitle as="p" class="capitalize">
                  <slot name="title" />
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
                <slot name="content" />

                <div v-if="type === 'alert'">
                  <VButton
                    class="w-full"
                    button-rank="tertiary"
                    button-type="neutral"
                    icon="x"
                    label="Close"
                    @click="emit('close')"
                  />
                </div>
                <div
                  v-else-if="type === 'save'"
                  class="py-3 flex justify-between"
                >
                  <VButton
                    button-rank="tertiary"
                    button-type="destructive"
                    icon="trash"
                    label="Cancel"
                    size="xs"
                    @click="emit('close')"
                  />
                  <VButton
                    :disabled="disableSave"
                    button-rank="primary"
                    button-type="success"
                    icon="plus-square"
                    label="Create"
                    size="xs"
                    @click="emit('save')"
                  />
                </div>
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
import { PropType, computed } from "vue";
import VButton from "@/molecules/VButton.vue";

const props = defineProps({
  open: { type: Boolean, default: false },
  size: {
    type: String as PropType<"sm" | "md" | "lg" | "xl" | "2xl">,
    required: true,
  },
  type: {
    type: String as PropType<"alert" | "save">,
    default: "alert",
  },
  disableSave: {
    type: Boolean,
    default: false,
  },
});

const dialogPanelClasses = computed(() => {
  let size;
  if (props.size === "sm") size = "max-w-sm";
  if (props.size === "md") size = "max-w-md";
  if (props.size === "lg") size = "max-w-lg";
  if (props.size === "xl") size = "max-w-xl";
  if (props.size === "2xl") size = "max-w-2xl";

  return `${size} w-full transform rounded bg-white dark:bg-neutral-900 text-left align-middle shadow-xl transition-all text-black dark:text-white`;
});

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save"): void;
}>();
</script>
