<template>
  <CollapsingGridItem
    ref="collapseRef"
    funcRunScreen
    disableScroll
    :disableCollapse="disableCollapse"
  >
    <template #header>
      <div class="flex flex-row items-center justify-between px-sm py-2xs">
        <h2 class="text-sm font-medium pt-xs pb-xs">{{ title }}</h2>
        <!-- Live updating indicator -->
        <div
          v-if="live"
          class="grow flex flex-row items-center text-xs text-action-400 ml-2"
        >
          <Icon name="loader" size="xs" class="mr-2xs" />
          Live
        </div>
      </div>
    </template>
    <div
      :class="
        clsx(
          'overflow-auto flex-1 text-sm',
          themeClasses('bg-white', 'bg-[#0d1117]'),
        )
      "
    >
      <slot />
    </div>
  </CollapsingGridItem>
</template>

<script setup lang="ts">
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, ref } from "vue";
import CollapsingGridItem from "./CollapsingGridItem.vue";
import { gridCollapseStyle } from "../util";

defineProps<{
  live: boolean;
  title: string;
  disableCollapse?: boolean;
}>();

const collapseRef = ref<InstanceType<typeof CollapsingGridItem>>();

const collapseStyle = computed(() => {
  if (collapseRef.value) {
    return gridCollapseStyle(collapseRef.value.openState.open);
  }
  return "1fr";
});

defineExpose({
  collapseStyle,
});
</script>
