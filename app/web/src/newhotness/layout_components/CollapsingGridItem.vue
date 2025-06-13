<template>
  <!-- TODO(Wendy) - I bet this and CollapsingFlexItem can be merged into one component -->
  <div :class="disableScroll ? 'overflow-hidden flex flex-col' : 'scrollable'">
    <h3
      :class="
        clsx(
          'group/header flex flex-row items-center',
          'sticky top-0 cursor-pointer text-lg font-bold px-xs',
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
    </h3>
    <slot />
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
