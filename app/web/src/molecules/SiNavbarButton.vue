<template>
  <Menu v-slot="{ open }" as="div" class="relative block h-full">
    <MenuButton
      v-tooltip.bottom="tooltipText"
      :class="buttonClasses(open)"
      :aria-label="props.tooltipText"
      class="relative"
      :disabled="disabled"
      @mouseenter="toggleHover"
      @mouseleave="toggleHover"
      @click="emit('click')"
    >
      <slot :hovered="hovered" :open="open"></slot>
    </MenuButton>

    <transition
      v-if="slots.dropdownContent"
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <SiDropdown :class="props.dropdownClasses">
        <slot name="dropdownContent"></slot>
      </SiDropdown>
    </transition>
  </Menu>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import { toRefs, useSlots } from "vue";
import { Menu } from "@headlessui/vue";
import SiDropdown from "@/molecules/SiDropdown.vue";
import { ref } from "vue";

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText: string;
  dropdownClasses?: string;
}>();

const { disabled } = toRefs(props);
const slots = useSlots();
const emit = defineEmits(["click"]);

const hovered = ref<boolean>(false);
const toggleHover = () => {
  hovered.value = !hovered.value;
};

const buttonClasses = (open: boolean) => {
  const results: Record<string, boolean> = {
    "h-full": true,
    "px-4": true,
    "hover:bg-black": true,
  };

  // Only display "selected" classes if there is a dropdown available
  // or we have explicitly passed in a selected value.
  if (props.selected || (open && slots.dropdownContent)) {
    results["hover:bg-black"] = false;
    results["bg-[#2F80ED]"] = true;
  }

  return results;
};
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
