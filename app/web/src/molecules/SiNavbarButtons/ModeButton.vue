<template>
  <Menu as="div" class="inline-block text-left">
    <MenuButton
      v-tooltip.bottom="tooltipText"
      :class="buttonClasses"
      :aria-label="props.tooltipText"
      :disabled="disabled"
      @mouseenter="toggleHover"
      @mouseleave="toggleHover"
      @click="emit('click')"
    >
      <slot :hovered="hovered"></slot>
    </MenuButton>
  </Menu>
</template>

<script setup lang="ts">
import { MenuButton } from "@headlessui/vue";
import { computed, toRefs } from "vue";
import { Menu } from "@headlessui/vue";
import { ref } from "vue";

const emit = defineEmits(["click"]);

const props = defineProps<{
  disabled?: boolean;
  selected?: boolean;
  tooltipText: string;
}>();
const { disabled, selected, tooltipText } = toRefs(props);

const hovered = ref<boolean>(false);
const toggleHover = () => {
  if (hovered.value) {
    hovered.value = false;
  } else {
    hovered.value = true;
  }
};

const selectedColor = "bg-[#2F80ED]";

const buttonClasses = computed(() => {
  const results: Record<string, boolean> = {
    "py-12": true,
    "px-4": true,
    "hover:bg-black": true,
  };

  if (selected?.value) {
    results["hover:bg-black"] = false;
    results[selectedColor] = true;
  }

  return results;
});
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
