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

      <div class="fixed inset-0 overflow-hidden">
        <div
          :class="
            clsx(
              'flex min-h-full items-center justify-center',
              !noWrapper && 'text-center',
            )
          "
        >
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
                  !noWrapper &&
                    'w-full rounded text-left align-middle shadow-2xl flex flex-col-reverse bg-white dark:bg-neutral-900 text-shade-100 dark:text-white',
                  'transform transition-all',
                  'max-h-full',
                  {
                    sm: 'max-w-sm',
                    md: 'max-w-md',
                    lg: 'max-w-lg',
                    xl: 'max-w-xl',
                    '2xl': 'max-w-2xl',
                    '4xl': 'max-w-4xl',
                    '4wxl': 'w-[56rem]',
                    '6xl': 'max-w-6xl',
                    '7xl': 'max-w-7xl',
                    max: 'max-w-[75vw]',
                  }[size],
                )
              "
              @click="onChromeClick"
            >
              <!-- fake input to prevent initial focus... only way to stop headless UI -->
              <input
                v-if="noAutoFocus"
                ref="noAutoFocusTrapRef"
                class="absolute w-0 h-0"
              />

              <div
                :class="
                  clsx(
                    'border-neutral-200 dark:border-shade-100 flex flex-col place-content-center text-sm',
                    !noInnerPadding && !noWrapper && 'p-sm',
                    !noWrapper && 'border-t',
                  )
                "
              >
                <slot />

                <div
                  v-if="type === 'save' && !noWrapper"
                  class="py-3 flex flex-row justify-between gap-sm"
                >
                  <VButton
                    buttonRank="tertiary"
                    icon="x"
                    label="Cancel"
                    size="xs"
                    tone="destructive"
                    variant="ghost"
                    @click="close"
                  />
                  <VButton
                    :disabled="disableSave"
                    :label="saveLabel"
                    class="grow"
                    icon="check"
                    size="xs"
                    tone="success"
                    @click="emit('save')"
                  />
                </div>
              </div>

              <div
                v-if="!noWrapper"
                :class="
                  clsx('flex justify-between items-center p-sm', titleClasses)
                "
              >
                <DialogTitle
                  :class="
                    clsx(
                      'font-medium line-clamp-5 pb-[1px]',
                      capitalizeTitle && 'capitalize',
                    )
                  "
                  as="p"
                >
                  <slot name="title">{{ title }}</slot>
                </DialogTitle>
                <div class="flex gap-xs items-center">
                  <slot name="titleIcons" />
                  <button
                    v-if="!noExit && !hideExitButton"
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
    type: String as PropType<
      "sm" | "md" | "lg" | "xl" | "2xl" | "4xl" | "4wxl" | "6xl" | "7xl" | "max"
    >,
    default: "md",
  },
  title: { type: String },
  capitalizeTitle: { type: Boolean, default: true },
  noExit: { type: Boolean },
  hideExitButton: { type: Boolean },
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
  noWrapper: Boolean,
  titleClasses: String,
  noInnerPadding: Boolean,
});

// make modal a new "theme container" but by passing no value, we reset the theme back to the root theme
// this makes sure things look right if the modal happens to be defined within a themed section
useThemeContainer();

const isOpen = ref(props.beginOpen);
function open() {
  hideFocusTrap.value = false;
  isOpen.value = true;
  setTimeout(fixAutoFocusElement);
}
function close() {
  emit("close");
  isOpen.value = false;
}

const focusTrapRef = ref();
const hideFocusTrap = ref(false);
const noAutoFocusTrapRef = ref<HTMLInputElement>();

const exitButtonRef = ref();
function fixAutoFocusElement() {
  // Headless UI automatically traps focus within the modal and focuses on the first focusable element it finds.
  // While focusing on an input (if there is one) feels good, focusing on an "OK" button or the close/X button
  // feels a bit agressive and looks strange
  const focusedEl = document.activeElement as HTMLElement;

  if (props.noAutoFocus) {
    focusedEl.blur();
    noAutoFocusTrapRef.value?.classList.add("hidden");
  }

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
  if (props.noExit) return;
  close();
}

const saveLabel = toRef(props, "saveLabel", "Create");

const onChromeClick = (e: MouseEvent) => {
  emit("click", e);
};

const emit = defineEmits<{
  close: [];
  closeComplete: [];
  save: [];
  click: [e: MouseEvent];
}>();

defineExpose({ open, close, isOpen });
</script>
