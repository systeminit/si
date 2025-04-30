<template>
  <div class="scrollable">
    <h3
      :class="
        clsx(
          'sticky top-0 cursor-pointer text-lg font-bold',
          themeClasses('bg-neutral-200', 'bg-neutral-900'),
        )
      "
      @click="openState.toggle"
    >
      <slot name="header" />
    </h3>
    <slot />
  </div>
</template>

<script lang="ts" setup>
import { themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useToggle } from "../logic_composables/toggle_containers";

const openState = useToggle();

defineExpose({
  openState,
});
</script>
