<template>
  <div
    :class="
      clsx(
        'flex flex-row flex-none items-center overflow-hidden text-shade-100 dark:text-shade-0',
        detailedListItem
          ? 'border-b border-neutral-200 dark:border-neutral-500'
          : 'border rounded h-[90px] cursor-pointer border-neutral-500 dark:hover:bg-action-700 hover:bg-action-100 dark:hover:outline-action-300 hover:outline-action-500 hover:outline -outline-offset-1',
      )
    "
  >
    <div class="flex flex-col gap-1 px-sm py-xs grow capsize overflow-hidden">
      <div
        :class="
          clsx(
            'text-md font-bold leading-tight',
            detailedListItem ? 'break-words' : 'truncate',
          )
        "
      >
        {{ secret.name }}
      </div>
      <div
        v-if="secret.updatedInfo"
        :class="
          clsx(
            'text-xs text-neutral-500 dark:text-neutral-300',
            !detailedListItem && 'truncate',
          )
        "
      >
        Updated:
        <Timestamp
          :date="new Date(secret.updatedInfo.timestamp)"
          :relative="!detailedListItem"
          :size="detailedListItem ? 'extended' : 'normal'"
        />
        by
        {{ secret.updatedInfo.actor.label }}
      </div>
      <div
        v-if="!secret.updatedInfo || detailedListItem"
        :class="
          clsx(
            'text-xs text-neutral-500 dark:text-neutral-300',
            !detailedListItem && 'truncate',
          )
        "
      >
        Created:
        <Timestamp
          :date="new Date(secret.createdInfo.timestamp)"
          :relative="!detailedListItem"
          :size="detailedListItem ? 'extended' : 'normal'"
        />
        by
        {{ secret.createdInfo.actor.label }}
      </div>
      <!-- TODO(Wendy) - eventually we will add expiry to secrets, here's the code to display it! -->
      <!-- <div
      v-if="detailedListItem"
      :class="
        clsx(
          'text-xs truncate',
          detailedListItem
            ? 'text-neutral-500 dark:text-neutral-300'
            : 'text-neutral-300',
        )
      "
    >
      Expires: {{ secret.expiration || "Never" }}
    </div> -->
      <div class="grow flex flex-row items-center">
        <div
          :class="
            clsx(
              'italic text-xs text-neutral-400',
              !detailedListItem && 'line-clamp-2',
            )
          "
        >
          <template v-if="secret.description">
            <span class="font-bold">Description:</span> {{ secret.description }}
          </template>
          <template v-else>No Description Available</template>
        </div>
      </div>
    </div>
    <div v-if="detailedListItem" class="pr-sm flex flex-col gap-xs">
      <IconButton
        icon="edit"
        tooltip="Edit"
        iconTone="action"
        iconIdleTone="neutral"
        @click="emit('edit')"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { Timestamp } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import clsx from "clsx";
import { Secret } from "../store/secrets.store";
import IconButton from "./IconButton.vue";

defineProps({
  secret: { type: Object as PropType<Secret>, required: true },
  detailedListItem: { type: Boolean },
});

const emit = defineEmits<{
  (e: "edit"): void;
}>();
</script>
