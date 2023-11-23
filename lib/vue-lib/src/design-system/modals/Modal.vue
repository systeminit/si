<template>
  <TransitionRoot
    :show="isOpen"
    appear
    as="template"
    @afterLeave="emit('closeComplete')"
  >
    <Dialog
      as="div"
      class="relative z-100"
      @close="exitHandler"
      @mousedown.stop
    >
      <TransitionChild
        as="template"
        enter="duration-300 ease-out"
        enterFrom="opacity-0"
        enterTo="opacity-100"
        leave="duration-200 ease-in"
        leaveFrom="opacity-100"
        leaveTo="opacity-0"
      >
        <div class="fixed inset-0 bg-shade-100 bg-opacity-60" />
      </TransitionChild>

      <div class="fixed inset-0 overflow-y-auto">
        <div class="flex min-h-full items-center justify-center text-center">
          <TransitionChild
            as="template"
            enter="duration-300 ease-out"
            enterFrom="opacity-0 scale-95"
            enterTo="opacity-100 scale-100"
            leave="duration-200 ease-in"
            leaveFrom="opacity-100 scale-100"
            leaveTo="opacity-0 scale-95"
          >
            <DialogPanel
              :class="
                clsx(
                  props.class,
                  'w-full rounded text-left align-middle shadow-2xl',
                  'flex flex-col-reverse',
                  'transform transition-all',
                  'bg-white dark:bg-neutral-900 text-shade-100 dark:text-white',
                  'max-h-full',
                  {
                    sm: 'max-w-sm',
                    md: 'max-w-md',
                    lg: 'max-w-lg',
                    xl: 'max-w-xl',
                    '2xl': 'max-w-2xl',
                    '4xl': 'max-w-4xl',
                    '6xl': 'max-w-6xl',
                  }[size],
                )
              "
            >
              <div
                class="p-sm border-t border-gray-200 dark:border-gray-900 flex flex-col place-content-center text-sm"
              >
                <slot />

                <div v-if="type === 'save'" class="py-3 flex justify-between">
                  <VButton
                    tone="destructive"
                    buttonRank="tertiary"
                    icon="trash"
                    label="Cancel"
                    size="xs"
                    @click="close"
                  />
                  <VButton
                    :disabled="disableSave"
                    tone="success"
                    icon="check"
                    :label="saveLabel"
                    size="xs"
                    @click="emit('save')"
                  />
                </div>
              </div>

              <div class="flex justify-between items-center p-sm">
                <DialogTitle
                  as="p"
                  :class="
                    clsx('capsize font-medium', capitalizeTitle && 'capitalize')
                  "
                >
                  <slot name="title">{{ title }}</slot>
                </DialogTitle>
                <button
                  v-if="!noExit"
                  ref="exitButtonRef"
                  :class="
                    clsx(
                      'modal-close-button',
                      'hover:scale-110 rounded-full opacity-80 hover:opacity-100 -mr-2 -my-2',
                    )
                  "
                  @click="close"
                >
                  <Icon name="x" size="md" />
                </button>
              </div>

              <!-- fake button to trap initial focus... only way to stop headless UI -->
              <button
                v-if="!hideFocusTrap"
                ref="focusTrapRef"
                class="absolute w-0 h-0"
              />
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
import { PropType, toRef, ref } from "vue";
import clsx from "clsx";
import { Icon, VButton } from "..";
import { useThemeContainer } from "../utils/theme_tools";

const props = defineProps({
  beginOpen: { type: Boolean, default: false },
  size: {
    type: String as PropType<"sm" | "md" | "lg" | "xl" | "2xl" | "4xl" | "6xl">,
    default: "md",
  },
  title: { type: String },
  capitalizeTitle: { type: Boolean, default: true },
  noExit: { type: Boolean },
  noClickOutExit: { type: Boolean },
  type: {
    type: String as PropType<"save" | "custom">,
  },
  disableSave: {
    type: Boolean,
    default: false,
  },
  saveLabel: {
    type: String,
    default: "Create",
    required: false,
  },
  noAutoFocus: Boolean,
  class: String,
});

// make modal a new "theme container" but by passing no value, we reset the theme back to the root theme
// this makes sure things look right if the modal happens to be defined within a themed section
useThemeContainer();

const isOpen = ref(props.beginOpen);
function open() {
  hideFocusTrap.value = false;
  isOpen.value = true;
  if (!props.noAutoFocus) {
    setTimeout(fixAutoFocusElement);
  }
}
function close() {
  emit("close");
  isOpen.value = false;
}

const focusTrapRef = ref();
const hideFocusTrap = ref(false);

const exitButtonRef = ref();
function fixAutoFocusElement() {
  // Headless UI automatically traps focus within the modal and focuses on the first focusable element it finds.
  // While focusing on an input (if there is one) feels good, focusing on an "OK" button or the close/X button
  // feels a bit agressive and looks strange
  const focusedEl = document.activeElement;
  if (
    focusedEl?.classList.contains("modal-close-button") ||
    focusedEl?.classList.contains("vbutton") ||
    focusTrapRef.value === focusedEl
  ) {
    // if we just blur focus, that first element will then be skipped in the tab order
    // instead of focus on a dummy button which will always be last in the order
    // so that hitting tab will focus on the correct first item
    focusTrapRef.value?.focus();
    focusTrapRef.value?.blur();
  }

  hideFocusTrap.value = true;
}

// "exit" triggered when clicking on background or hitting escape key
// (currently this is done via headless UI)
function exitHandler() {
  if (props.noExit || props.noClickOutExit) return;
  close();
}

const saveLabel = toRef(props, "saveLabel", "Create");

const emit = defineEmits<{
  close: [];
  closeComplete: [];
  save: [];
}>();

defineExpose({ open, close, isOpen });
</script>
