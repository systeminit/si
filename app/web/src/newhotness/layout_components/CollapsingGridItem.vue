<template>
  <div class="overflow-hidden min-h-0 flex flex-col">
    <h3
      :class="
        clsx(
          'group/header flex flex-row items-center h-[40px]',
          'cursor-pointer text-lg font-bold px-xs py-xs flex-none border',
          themeClasses(
            'bg-neutral-300 hover:bg-neutral-400 border-neutral-400',
            'bg-neutral-800 hover:bg-neutral-700 border-neutral-600',
          ),
        )
      "
      @click="openState.toggle"
    >
      <Icon
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
        disableScroll
          ? 'overflow-hidden flex flex-col min-h-[calc(100%-40px)]'
          : 'scrollable flex-1 min-h-0'
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

defineProps({
  disableScroll: { type: Boolean },
});

const openState = useToggle();

defineExpose({
  openState,
});
</script>
