<template>
  <component
    :is="linkTo ? RouterLink : 'button'"
    v-tooltip="{
      content: tooltipText,
      theme: 'instant-show',
    }"
    :to="linkTo"
    :class="
      clsx(
        'relative h-full flex flex-row items-center children:pointer-events-none p-sm',
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
    <Icon v-if="icon" :name="icon" />

    <DropdownMenu v-if="slots.dropdownContent" ref="dropdownRef">
      <slot name="dropdownContent" />
    </DropdownMenu>
    <DropdownMenu v-if="slots.dropdownContentSecondary" ref="dropdownSecondaryRef">
      <slot name="dropdownContentSecondary" />
    </DropdownMenu>
  </component>
</template>

<script lang="ts" setup>
import { computed, ref, useSlots, PropType } from "vue";
import { RouterLink } from "vue-router";
import clsx from "clsx";
import { DropdownMenu, IconNames, Icon } from "@si/vue-lib/design-system";

const props = defineProps({
  selected: { type: Boolean },
  tooltipText: { type: String },
  linkTo: { type: [String, Object] },
  externalLinkTo: { type: String },
  icon: { type: String as PropType<IconNames> },
});

const dropdownRef = ref<InstanceType<typeof DropdownMenu>>();
const dropdownSecondaryRef = ref<InstanceType<typeof DropdownMenu>>();

const slots = useSlots();
const emit = defineEmits(["click"]);

const hovered = ref<boolean>(false);
const toggleHover = () => {
  hovered.value = !hovered.value;
};

const isSelectedOrMenuOpen = computed(() => {
  if (props.selected) return true;
  if (!dropdownRef.value) return false;
  return dropdownRef.value.isOpen || dropdownSecondaryRef.value?.isOpen;
});

function onClick(e: MouseEvent) {
  if (dropdownRef.value) {
    openEvent.value = e;
    dropdownRef.value.open(e);
  }
  if (props.externalLinkTo) {
    window.open(props.externalLinkTo, "_blank");
  }
  emit("click", e);
}

const openEvent = ref<MouseEvent | undefined>();

const openSecondary = () => {
  dropdownSecondaryRef.value?.open(openEvent.value);
};

defineExpose({ openSecondary });
</script>
