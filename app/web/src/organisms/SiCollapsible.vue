<template>
  <Disclosure v-slot="{ open }" :as="as" :default-open="defaultOpen">
    <DisclosureButton :class="disclosureButtonClasses">
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
    </DisclosureButton>
    <DisclosurePanel :as="contentAs">
      <slot />
    </DisclosurePanel>
  </Disclosure>
</template>

<script setup lang="ts">
import { computed, useSlots, PropType } from "vue";
import { Disclosure, DisclosurePanel, DisclosureButton } from "@headlessui/vue";
import Icon from "@/ui-lib/Icon.vue";
import { ThemeValue } from "@/observable/theme";

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
  hideBottomBorder: { type: Boolean, default: false },
  forceTheme: { type: String as PropType<ThemeValue> },
});

const disclosureButtonClasses = computed(() => {
  let classes = `${props.buttonClasses} flex w-full px-2 py-2 text-left font-medium focus:outline-none items-center`;

  if (!props.hideBottomBorder) {
    if (props.forceTheme === "dark") {
      classes += " border-neutral-600 border-b";
    } else if (props.forceTheme === "light") {
      classes += " border-neutral-200 border-b";
    } else {
      classes += " border-neutral-200 dark:border-neutral-600 border-b";
    }
  }

  if (props.textSize === "sm") return `${classes} text-sm`;
  else if (props.textSize === "lg") return `${classes} text-lg`;
  else return `${classes} text-base`;
});

const slots = useSlots();
const labelSlot = computed(() => slots.label?.());
</script>
