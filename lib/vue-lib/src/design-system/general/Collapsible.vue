<template>
  <Disclosure v-slot="{ open }" :as="as" :default-open="defaultOpen">
    <DisclosureButton
      :class="
        clsx(
          buttonClasses,
          'flex w-full px-2 py-2 text-left font-medium focus:outline-none items-center',
          !hideBottomBorder &&
            !(hideBottomBorderWhenOpen && open) && [
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
    >
      <slot name="prefix" />
      <Icon
        :name="open ? 'chevron--down' : 'chevron--right'"
        size="sm"
        class="mr-1.5 dark:text-white flex-shrink-0 block"
      />

      <slot name="label" />
      <span
        v-if="labelSlot === undefined || showLabelAndSlot"
        class="whitespace-nowrap overflow-hidden overflow-ellipsis"
      >
        {{ label }}
      </span>

      <div v-if="$slots.right" class="flex-shrink-0">
        <slot name="right" />
      </div>
    </DisclosureButton>
    <DisclosurePanel :as="contentAs">
      <slot />
    </DisclosurePanel>
  </Disclosure>
</template>

<script setup lang="ts">
import { computed, useSlots, PropType } from "vue";
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
import clsx from "clsx";
import { Icon, themeClasses } from "..";

const props = defineProps({
  label: { type: String },
  showLabelAndSlot: { type: Boolean, default: false },
  as: { type: String },
  contentAs: { type: String },
  buttonClasses: { type: String, default: "" },
  defaultOpen: { type: Boolean, default: true },
  textSize: {
    type: String as PropType<"sm" | "md" | "lg">,
    default: "sm",
  },
  hideBottomBorderWhenOpen: { type: Boolean, default: false },
  hideBottomBorder: { type: Boolean, default: false },
});

const slots = useSlots();
const labelSlot = computed(() => slots.label?.());
</script>
