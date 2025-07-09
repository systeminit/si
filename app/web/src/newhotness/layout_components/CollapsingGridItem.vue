<template>
  <div class="overflow-hidden min-h-0 flex flex-col">
    <h3
      :class="
        clsx(
          'group/header flex flex-row items-center h-[28px]',
          'cursor-pointer text-lg font-bold px-xs flex-none',
          themeClasses(
            'bg-neutral-200 hover:bg-neutral-300',
            'bg-neutral-900 hover:bg-black',
          ),
        )
      "
      @click="openState.toggle"
    >
      <Icon
        class="group-hover/header:scale-125"
        :name="openState.open.value ? 'chevron--down' : 'chevron--right'"
      />
      <slot name="header" />
      <div class="ml-auto" />
      <slot name="headerIconsRight" />
    </h3>
    <div
      :class="
        disableScroll
          ? 'overflow-hidden flex flex-col min-h-[calc(100%-28px)]'
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
