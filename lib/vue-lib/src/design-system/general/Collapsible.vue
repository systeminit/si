<template>
  <div>
    <div
      :class="
        clsx(
          buttonClasses,
          'flex w-full px-2 py-2 text-left font-medium focus:outline-none items-center cursor-pointer',
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
      @click="toggleIsOpen"
    >
      <slot name="prefix" />
      <Icon
        :name="isOpen ? 'chevron--down' : 'chevron--right'"
        size="sm"
        class="mr-1.5 dark:text-white flex-shrink-0 block"
      />

      <slot name="label" />
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
import { PropType, ref } from "vue";
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
});

const isOpen = ref(props.defaultOpen);

function toggleIsOpen() {
  isOpen.value = !isOpen.value;
}

defineExpose({ isOpen });
</script>
