<template>
  <div
    :class="
      clsx(
        'flex flex-col rounded border',
        // basis-1/3
        themeClasses(
          'border-neutral-200 bg-shade-0',
          'border-neutral-700 bg-neutral-800',
        ),
      )
    "
  >
    <div
      :class="
        clsx(
          'flex flex-row items-center gap-xs p-xs border-b',
          themeClasses('border-neutral-200', 'border-neutral-700'),
        )
      "
    >
      <div v-if="title" class="text-lg font-bold flex-grow">{{ title }}</div>
      <slot v-else name="title" />
      <a v-if="helpLink" :href="helpLink" target="_blank">
        <Icon
          v-tooltip="helpTooltipText"
          name="question-circle"
          class="flex-none cursor-pointer"
        />
      </a>
      <Icon
        v-else
        v-tooltip="helpTooltipText"
        name="question-circle"
        class="flex-none cursor-pointer"
      />
    </div>
    <div class="flex flex-col p-xs">
      <div class="text-xl font-bold"><slot name="infoRow1" /></div>
      <div class="text-xs"><slot name="infoRow2" /></div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";

defineProps({
  title: { type: String },
  helpTooltipText: {
    type: String,
    default: "",
  },
  helpLink: {
    type: String,
  },
});
</script>
