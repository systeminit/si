<template>
  <component
    :is="linkTo ? RouterLink : 'button'"
    v-tooltip="{
      content: tooltipText,
      delay: { show: 10, hide: 100 },
      instantMove: true,
    }"
    :to="linkTo"
    :class="
      clsx(
        'relative flex items-center children:pointer-events-none px-sm',
        isSelectedOrMenuOpen && 'bg-action-500',
        !isSelectedOrMenuOpen && 'hover:bg-black',
      )
    "
    :aria-label="tooltipText"
    @click="onClick"
    @mouseenter="toggleHover"
    @mouseleave="toggleHover"
  >
    <slot :open="isSelectedOrMenuOpen" :hovered="hovered" />

    <DropdownMenu v-if="slots.dropdownContent" ref="dropdownRef">
      <slot name="dropdownContent" />
    </DropdownMenu>
  </component>
</template>

<script lang="ts" setup>
import { computed, ref, useSlots } from "vue";
import { RouterLink } from "vue-router";
import clsx from "clsx";
import { DropdownMenu } from "@si/vue-lib/design-system";

const props = defineProps({
  selected: { type: Boolean },
  tooltipText: { type: String },
  linkTo: { type: [String, Object] },
});

const dropdownRef = ref<InstanceType<typeof DropdownMenu>>();

const slots = useSlots();
const emit = defineEmits(["click"]);

const hovered = ref<boolean>(false);
const toggleHover = () => {
  hovered.value = !hovered.value;
};

const isSelectedOrMenuOpen = computed(() => {
  if (props.selected) return true;
  if (!dropdownRef.value) return false;
  return dropdownRef.value.isOpen;
});

function onClick(e: MouseEvent) {
  if (dropdownRef.value) dropdownRef.value.open(e);
  emit("click", e);
}
</script>
