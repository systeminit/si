<template>
  <TransitionRoot :show="isOpen" appear as="template">
    <Dialog
      as="div"
      class="relative z-50"
      :initial-focus="initialFocusTrapRef"
      @close="exitHandler"
    >
      <TransitionChild
        as="template"
        enter="duration-300 ease-out"
        enter-from="opacity-0"
        enter-to="opacity-100"
        leave="duration-200 ease-in"
        leave-from="opacity-100"
        leave-to="opacity-0"
      >
        <div class="fixed inset-0 bg-shade-100 bg-opacity-60" />
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
              :class="
                clsx(
                  'w-full rounded text-left align-middle shadow-2xl',
                  'flex flex-col-reverse',
                  'transform transition-all',
                  'bg-white dark:bg-neutral-900 text-shade-100 dark:text-white',
                  {
                    sm: 'max-w-sm',
                    md: 'max-w-md',
                    lg: 'max-w-lg',
                    xl: 'max-w-xl',
                    '2xl': 'max-w-2xl',
                  }[size],
                )
              "
            >
              <!-- fake button to trap initial focus... only way to stop headless UI -->
              <button
                v-if="!hideInitialFocusTrap"
                ref="initialFocusDummyRef"
                class="absolute w-0 h-0"
                @blur="hideInitialFocusTrap = true"
              />

              <div
                class="p-sm border-t border-gray-200 dark:border-gray-900 flex flex-col place-content-center text-sm"
              >
                <slot />

                <div v-if="type === 'save'" class="py-3 flex justify-between">
                  <VButton
                    button-rank="tertiary"
                    button-type="destructive"
                    icon="trash"
                    label="Cancel"
                    size="xs"
                    @click="close"
                  />
                  <VButton
                    :disabled="disableSave"
                    button-rank="primary"
                    button-type="success"
                    icon="check"
                    :label="saveLabel"
                    size="xs"
                    @click="emit('save')"
                  />
                </div>
              </div>

              <div class="flex justify-between items-center p-sm">
                <DialogTitle as="p" class="capitalize capsize font-medium">
                  <slot name="title">{{ title }}</slot>
                </DialogTitle>
                <button
                  v-if="!hideTopCloseButton"
                  class="hover:scale-110 rounded-full opacity-80 hover:opacity-100 -mr-2 -my-2"
                  @click="close"
                >
                  <Icon name="x" size="md" />
                </button>
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
import { PropType, computed, toRef, ref } from "vue";
import clsx from "clsx";
import VButton from "@/molecules/VButton.vue";
import SiButtonIcon from "@/atoms/SiButtonIcon.vue";
import Icon from "../icons/Icon.vue";
import VButton2 from "../VButton2.vue";

const props = defineProps({
  beginOpen: { type: Boolean, default: false },
  size: {
    type: String as PropType<"sm" | "md" | "lg" | "xl" | "2xl">,
    default: "md",
    required: true,
  },
  title: { type: String },
  type: {
    type: String as PropType<"save" | "custom">,
  },
  disableSave: {
    type: Boolean,
    default: false,
  },
  disableExit: {
    type: Boolean,
    default: false,
  },
  hideTopCloseButton: {
    type: Boolean,
    default: false,
  },
  saveLabel: {
    type: String,
    default: "Create",
    required: false,
  },
});

const hideInitialFocusTrap = ref(false);
const initialFocusTrapRef = ref();

const isOpen = ref(props.beginOpen);
function open() {
  hideInitialFocusTrap.value = false;
  isOpen.value = true;
}
function close() {
  emit("close");
  isOpen.value = false;
}

// "exit" triggered when clicking on background or hitting escape key
// (currently this is done via headless UI)
function exitHandler() {
  if (props.disableExit) return;
  close();
}

const saveLabel = toRef(props, "saveLabel", "Create");

const emit = defineEmits<{
  (e: "close"): void;
  (e: "save"): void;
}>();

defineExpose({ open, close, isOpen });
</script>
