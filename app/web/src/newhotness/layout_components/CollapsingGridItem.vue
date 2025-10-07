<template>
  <div class="overflow-hidden min-h-0 flex flex-col">
    <h3
      :class="
        clsx(
          'group/header flex flex-row items-center h-[40px]',
          'text-lg font-bold px-xs py-xs flex-none border',
          themeClasses(
            'bg-neutral-300 border-neutral-400',
            'bg-neutral-800 border-neutral-600',
          ),
          !disableCollapse && [
            'cursor-pointer',
            themeClasses('hover:bg-neutral-400', 'hover:bg-neutral-700'),
          ],
        )
      "
      @click="toggleOpen"
    >
      <Icon
        v-if="!disableCollapse"
        class="group-hover/header:scale-125 mr-xs"
        :name="openState.open.value ? 'chevron-down' : 'chevron-right'"
        size="sm"
      />
      <slot name="header" />
      <div class="ml-auto" />
      <slot name="headerIconsRight" />
    </h3>
    <div
      :class="
        clsx(
          disableScroll
            ? 'overflow-hidden flex flex-col min-h-[calc(100%-40px)]'
            : 'scrollable flex-1 min-h-0',
          funcRunScreen && [
            'border-x border-b',
            themeClasses(
              'bg-neutral-300 border-neutral-400',
              'bg-neutral-800 border-neutral-600',
            ),
          ],
        )
      "
    >
      <slot />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useToggle } from "../logic_composables/toggle_containers";

const props = defineProps({
  disableScroll: { type: Boolean },
  disableCollapse: { type: Boolean },
  funcRunScreen: { type: Boolean },
});

const openState = useToggle();
const toggleOpen = () => {
  if (props.disableCollapse) return;
  openState.toggle();
};

defineExpose({
  openState,
});
</script>
