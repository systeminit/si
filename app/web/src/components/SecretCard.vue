<template>
  <div
    :class="
      clsx(
        'flex flex-col border-b border-neutral-200 dark:border-neutral-600 p-xs gap-1 cursor-pointer',
        'hover:outline-blue-300 hover:outline -outline-offset-1 hover:bg-action-100 hover:dark:bg-action-800 hover:rounded',
      )
    "
  >
    <div class="text-md truncate font-bold text-shade-100 dark:text-shade-0">
      {{ secret.name }}
    </div>
    <div class="text-xs text-neutral-500 dark:text-neutral-300 truncate">
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
    <div
      v-if="secret.description"
      class="italic text-xs line-clamp-2 text-neutral-400"
    >
      <span class="font-bold">Description:</span> {{ secret.description }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { Timestamp } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import clsx from "clsx";
import { Secret } from "../store/secrets.store";

defineProps({
  secret: { type: Object as PropType<Secret>, required: true },
});
</script>
