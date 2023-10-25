<template>
  <div
    :class="
      clsx(
        themeContainerClasses,
        'flex flex-row flex-none items-center border-b overflow-hidden',
        detailedListItem
          ? 'border-neutral-200 dark:border-neutral-500 text-shade-100 dark:text-shade-0'
          : 'h-[90px] cursor-pointer border-neutral-500 text-shade-0 hover:bg-action-800 hover:outline-blue-300 hover:outline -outline-offset-1  hover:rounded',
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
            'text-xs',
            detailedListItem
              ? 'text-neutral-500 dark:text-neutral-300'
              : 'text-neutral-300 truncate',
          )
        "
      >
        > Updated:
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
            'text-xs',
            detailedListItem
              ? 'text-neutral-500 dark:text-neutral-300'
              : 'text-neutral-300 truncate',
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
    <div
      v-if="detailedListItem"
      class="pr-sm flex flex-col gap-xs self-stretch"
    >
      <!-- TODO(Wendy) - this button is a mock, will wire it up soon! -->
      <div
        :class="
          clsx(
            'grow flex items-center cursor-pointer',
            themeClasses(
              'hover:text-action-500 text-neutral-400',
              'hover:text-action-400',
            ),
          )
        "
        @click="emit('edit')"
      >
        <Icon name="settings" size="lg" />
      </div>
      <!-- TODO(Wendy) - here's the button we will use when we add deletion -->
      <!-- <div
        :class="
          clsx(
            'grow flex items-center cursor-pointer hover:text-destructive-500',
            themeClasses('text-neutral-400', ''),
          )
        "
      >
        <Icon name="trash" size="lg" />
      </div> -->
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  Icon,
  Timestamp,
  themeClasses,
  useThemeContainer,
} from "@si/vue-lib/design-system";
import { PropType } from "vue";
import clsx from "clsx";
import { Secret } from "../store/secrets.store";

const { themeContainerClasses } = useThemeContainer("dark");

defineProps({
  secret: { type: Object as PropType<Secret>, required: true },
  detailedListItem: { type: Boolean },
});

const emit = defineEmits<{
  (e: "edit"): void;
}>();
</script>
