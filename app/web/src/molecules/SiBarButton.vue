<template>
  <component
    :is="routerLinkTo ? RouterLink : 'button'"
    v-if="!$slots.dropdownContent || routerLinkTo"
    v-tooltip.bottom="tooltipText"
    :to="routerLinkTo"
    class="relative block h-full flex items-center"
    :class="buttonClasses(false)"
    :aria-label="props.tooltipText"
    @click="emit('click')"
  >
    <slot />
  </component>
  <Menu v-else v-slot="{ open }" as="div" class="relative block h-full">
    <MenuButton
      v-tooltip.bottom="tooltipText"
      :aria-label="props.tooltipText"
      :class="buttonClasses(open)"
      :disabled="disabled"
      class="relative"
      @mouseenter="toggleHover"
      @mouseleave="toggleHover"
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
      <SiDropdown :class="props.dropdownClasses" :navbar="navbar">
        <slot name="dropdownContent"></slot>
      </SiDropdown>
    </transition>
  </Menu>
</template>

<script lang="ts" setup>
import { Menu, MenuButton } from "@headlessui/vue";
import { provide, ref, toRefs, useSlots } from "vue";
import SiDropdown from "@/molecules/SiDropdown.vue";
import { RouterLink } from "vue-router";

const props = withDefaults(
  defineProps<{
    disabled?: boolean;
    selected?: boolean;
    tooltipText?: string;
    dropdownClasses?: string;
    paddingX?: number;
    hoverEffect?: boolean;

    // Fills the entire width with the button (including the selected and hover colors).
    // It is recommended that the item in the slot uses the "flex flex-row justify-center" classes in
    // conjunction with this prop.
    fillEntireWidth?: boolean;
    dropdownItemClasses?: string;
    dropdownItemShowPrefix?: boolean;
    dropdownItemShowSuffix?: boolean;
    navbar?: boolean;
    routerLinkTo?: object;
  }>(),
  {
    paddingX: 4,
    hoverEffect: true,
    tooltipText: "",
    dropdownItemClasses: "text-center",
    dropdownItemShowPrefix: true,
    dropdownItemShowSuffix: true,
    navbar: true,
  },
);

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
  };

  if (props.paddingX > 0) {
    results[`px-${props.paddingX}`] = true;
  }

  if (props.hoverEffect) {
    results["hover:bg-black"] = true;
  }

  if (props.fillEntireWidth) {
    results["w-full"] = true;
  }

  // Only display "selected" classes if there is a dropdown available
  // or we have explicitly passed in a selected value.
  if (props.selected || (open && slots.dropdownContent)) {
    results["hover:bg-black"] = false;
    results["bg-action-500"] = true;
  }

  return results;
};

provide("dropdownItemClasses", props.dropdownItemClasses);
provide("dropdownItemShowPrefix", props.dropdownItemShowPrefix);
provide("dropdownItemShowSuffix", props.dropdownItemShowSuffix);
</script>

<style lang="scss" scoped>
.cursor-not-allowed {
  cursor: not-allowed;
}
</style>
