<template>
  <div
    class="flex flex-row items-center"
    :class="
      isSelected
        ? [
            'bg-action-100 dark:bg-action-700 border border-action-500 dark:border-action-300',
          ]
        : [
            'border border-transparent hover:border-action-500 dark:hover:border-action-300',
          ]
    "
  >
    <slot />

    <Icon :name="icon" />

    <span
      :class="isSelected ? ['bg-action-100 dark:bg-action-700'] : []"
      class="px-2 py-2 group flex flex-col items-baseline w-full"
    >
      <span
        class="whitespace-nowrap text-ellipsis overflow-hidden shrink leading-tight"
        >{{ title || "si-123" }}</span
      >
      <i
        :class="
          isSelected
            ? ['bg-action-100 dark:bg-action-700']
            : ['text-neutral-500']
        "
        class="text-sm flex-none"
      >
        {{ subtitle }}
      </i>
    </span>
    <div
      class="flex flex-row items-center gap-1.5 pl-4 pr-3 border-l dark:border-neutral-600"
    >
      <Icon
        v-if="statusIcons.change"
        :name="statusIcons.change.icon"
        :tone="statusIcons.change.tone"
      />
      <div v-else class="w-6 h-6" />
      <Icon
        v-if="statusIcons.qualification"
        :name="statusIcons.qualification.icon"
        :tone="statusIcons.qualification.tone"
      />
      <div v-else class="w-6 h-6" />
      <Icon
        v-if="statusIcons.confirmation"
        :name="statusIcons.confirmation.icon"
        :tone="statusIcons.confirmation.tone"
      />
      <div v-else class="w-6 h-6" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ComponentTreeNode, StatusIconsSet } from "@/store/components.store";
import Icon from "@/ui-lib/icons/Icon.vue";
import { IconNames } from "@/ui-lib/icons/icon_set";

const props = defineProps<{
  title: string;
  subtitle: string;
  isSelected: boolean;
  statusIcons: StatusIconsSet;
  icon: IconNames;
}>();
</script>
