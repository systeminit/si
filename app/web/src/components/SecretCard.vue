<template>
  <div
    :class="
      clsx(
        themeContainerClasses,
        'flex flex-col border-b px-sm py-xs gap-1 cursor-pointer h-[90px] flex-none',
        'hover:outline-blue-300 hover:outline -outline-offset-1  hover:rounded',
        // 'border-neutral-200 dark:border-neutral-500 text-shade-100 dark:text-shade-0 hover:bg-action-100 hover:dark:bg-action-800', // dark/light mode classes
        'border-neutral-500 text-shade-0 hover:bg-action-800', // force dark mode classes
      )
    "
  >
    <div class="text-md truncate font-bold">
      {{ secret.name }}
    </div>
    <div
      :class="
        clsx(
          'text-xs truncate',
          // 'text-neutral-500 dark:text-neutral-300', // dark/light mode classes
          'text-neutral-300', // force dark mode class
        )
      "
    >
      <template v-if="secret.updatedInfo">
        Updated:
        <Timestamp :date="new Date(secret.updatedInfo.timestamp)" relative /> by
        {{ secret.updatedInfo.actor.label }}
      </template>
      <template v-else>
        Created:
        <Timestamp :date="new Date(secret.createdInfo.timestamp)" relative /> by
        {{ secret.createdInfo.actor.label }}
      </template>
    </div>
    <div class="grow flex flex-row items-center">
      <div class="italic text-xs line-clamp-2 text-neutral-400">
        <template v-if="secret.description">
          <span class="font-bold">Description:</span> {{ secret.description }}
        </template>
        <template v-else>No Description Available</template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Timestamp, useThemeContainer } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import clsx from "clsx";
import { Secret } from "../store/secrets.store";

const { themeContainerClasses } = useThemeContainer("dark");

defineProps({
  secret: { type: Object as PropType<Secret>, required: true },
});
</script>
