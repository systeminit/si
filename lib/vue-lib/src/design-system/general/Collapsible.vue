<template>
  <div
    :class="
      clsx(
        isOpen && extraBorderAtBottomOfContent && 'border-b',
        themeClasses('border-neutral-200', 'border-neutral-600'),
      )
    "
  >
    <div
      :class="
        clsx(
          buttonClasses,
          'flex w-full py-xs text-left font-medium focus:outline-none items-center cursor-pointer',
          xPaddingClass,
          !hideBottomBorder &&
            !(hideBottomBorderWhenOpen && isOpen) && [
              'border-b',
              themeClasses('border-neutral-200', 'border-neutral-600'),
            ],
          {
            sm: 'text-sm',
            md: 'text-base',
            lg: 'text-lg',
          }[textSize],
        )
      "
      @click="toggleIsOpen()"
    >
      <slot name="prefix" />
      <Icon
        :name="isOpen ? 'chevron--down' : 'chevron--right'"
        size="sm"
        class="mr-1.5 dark:text-white flex-shrink-0 block"
      />

      <span
        v-if="
          (!$slots.openLabel || showLabelAndSlot) &&
          isOpen &&
          useDifferentLabelWhenOpen
        "
        class="whitespace-nowrap overflow-hidden overflow-ellipsis"
      >
        {{ label }}
      </span>
      <slot v-if="isOpen && useDifferentLabelWhenOpen" name="openLabel" />
      <slot v-else name="label" />
      <span
        v-if="!$slots.label || showLabelAndSlot"
        class="whitespace-nowrap overflow-hidden overflow-ellipsis"
      >
        {{ label }}
      </span>

      <div v-if="$slots.right" class="flex-shrink-0">
        <slot name="right" />
      </div>
    </div>

    <div v-if="isOpen" :class="contentClasses">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { PropType, ref, computed } from "vue";
import clsx from "clsx";
import { Icon, themeClasses } from "..";

const props = defineProps({
  label: { type: String },
  showLabelAndSlot: { type: Boolean, default: false },
  as: { type: String },
  contentAs: { type: String },
  buttonClasses: { type: String, default: "" },
  contentClasses: { type: String },
  defaultOpen: { type: Boolean, default: true },
  textSize: {
    type: String as PropType<"sm" | "md" | "lg">,
    default: "sm",
  },
  hideBottomBorderWhenOpen: { type: Boolean, default: false },
  hideBottomBorder: { type: Boolean, default: false },
  extraBorderAtBottomOfContent: { type: Boolean, default: false },
  useDifferentLabelWhenOpen: { type: Boolean },
  openLabel: { type: String },
  xPadding: {
    type: String as PropType<"none" | "standard" | "double">,
    default: "standard",
  },
});

const xPaddingClass = computed(() => {
  if (props.xPadding === "standard") return "px-xs";
  else if (props.xPadding === "double") return "px-sm";
  else return "";
});

const isOpen = ref(props.defaultOpen);

function toggleIsOpen(open?: boolean) {
  if (open === undefined) {
    isOpen.value = !isOpen.value;
  } else {
    isOpen.value = open;
  }
}

defineExpose({ isOpen, toggleIsOpen });
</script>
