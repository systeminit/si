<template>
  <Menu as="div" class="inline-block text-left">
    <div :class="bgClasses">
      <MenuButton
        v-tooltip.bottom="tooltipText"
        :class="buttonClasses"
        :aria-label="props.tooltipText"
        :disabled="disabled"
        @click="emit('click')"
      >
        <slot></slot>
      </MenuButton>
    </div>

    <transition
      v-if="enableDropdown && props.options"
      enter-active-class="transition ease-out duration-100"
      enter-from-class="transform opacity-0 scale-95"
      enter-to-class="transform opacity-100 scale-100"
      leave-active-class="transition ease-in duration-75"
      leave-from-class="transform opacity-100 scale-100"
      leave-to-class="transform opacity-0 scale-95"
    >
      <SiIconDropdown
        :options="props.options"
        :menu-items-class="'origin-top-right absolute right-0 mt-2 w-56 rounded-md shadow-lg bg-white ring-1 ring-black ring-opacity-5 focus:outline-none'"
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

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText: string;
  panelSwitcher?: boolean;
  options?: SiIconDropdownOption[];
}>();
const { disabled, selected, tooltipText } = toRefs(props);

const enableDropdown = computed((): boolean => {
  if (props.options && props.options.length > 0) {
    return true;
  }
  return false;
});

const selectedPanelBgColor = "bg-[#2F80ED]";
const selectedBgColor = "bg-black";

const bgClasses = computed(() => {
  const results: Record<string, boolean> = {
    "py-12": true,
    "px-4": true,
    "hover:bg-black": true,
  };

  if (selected?.value) {
    results["hover:bg-black"] = false;
    if (props.panelSwitcher) {
      results[selectedPanelBgColor] = true;
    } else {
      results[selectedBgColor] = true;
    }
  }

  return results;
});

const buttonClasses = computed(() => {
  const results: Record<string, boolean> = {
    block: true,
    "w-6": true,
    "h-6": true,
    "text-gray-300": true,
    "hover:text-white": true,
  };
  if (disabled?.value) {
    results["opacity-50"] = true;
    results["cursor-not-allowed"] = true;
  } else if (selected?.value) {
    results["text-white"] = true;
    results["text-gray-300"] = false;
    results["hover:text-white"] = false;
  }
  return results;
});
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
