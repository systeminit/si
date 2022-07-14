<template>
  <Menu v-slot="{ open }" as="div" class="inline-block relative text-left">
    <MenuButton
      v-tooltip.bottom="tooltipText"
      :class="buttonClasses(open)"
      :aria-label="props.tooltipText"
      class="relative"
      :disabled="disabled"
      @mouseenter="toggleHover"
      @mouseleave="toggleHover"
    >
      <slot :hovered="hovered" :open="open"></slot>
    </MenuButton>

    <transition
      v-if="enableDropdown && props.options"
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95 rounded-md"
      enter-to-class="transform opacity-100 scale-100 rounded-md"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100 rounded-md"
      leave-to-class="transform opacity-0 scale-95 rounded-md"
    >
      <SiIconDropdown
        :options="props.options"
        class="min-w-full"
      ></SiIconDropdown>
    </transition>
  </Menu>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import { computed, toRefs } from "vue";
import { Menu } from "@headlessui/vue";
import SiIconDropdown from "@/atoms/SiIconDropdown.vue";
import { SiIconDropdownOption } from "@/atoms/SiIconDropdown/types";
import { ref } from "vue";

const props = defineProps<{
  disabled?: boolean;
  tooltipText: string;
  options?: SiIconDropdownOption[];
}>();
const { disabled, tooltipText } = toRefs(props);

const enableDropdown = computed((): boolean => {
  if (props.options && props.options.length > 0) {
    return true;
  }
  return false;
});

const hovered = ref<boolean>(false);
const toggleHover = () => {
  if (hovered.value) {
    hovered.value = false;
  } else {
    hovered.value = true;
  }
};

const buttonClasses = (open: boolean) => {
  const results: Record<string, boolean> = {
    "py-12": true,
    "px-4": true,
    "hover:bg-black": true,
  };

  // Only display "selected" classes if there is a dropdown available.
  if (open && enableDropdown.value) {
    results["hover:bg-black"] = false;
    results["bg-black"] = true;
  }

  return results;
};
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
