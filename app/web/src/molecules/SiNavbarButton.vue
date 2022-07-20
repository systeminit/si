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
      v-if="enableDropdown && props.options"
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95 rounded-md"
      enter-to-class="transform opacity-100 scale-100 rounded-md"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100 rounded-md"
      leave-to-class="transform opacity-0 scale-95 rounded-md"
    >
      <SiDropdown
        :options="props.options"
        class="min-w-full"
        :class="dropdownClasses"
      />
    </transition>
  </Menu>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import { computed, toRefs } from "vue";
import { Menu } from "@headlessui/vue";
import SiDropdown from "@/atoms/SiDropdown.vue";
import { SiDropdownOption } from "@/atoms/SiDropdown.vue";
import { ref } from "vue";

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText: string;
  options?: SiDropdownOption[];
  dropdownClasses?: string;
}>();
const { disabled } = toRefs(props);

const enableDropdown = computed((): boolean => {
  return !!props.options?.length;
});

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
  if (props.selected || (open && enableDropdown.value)) {
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
