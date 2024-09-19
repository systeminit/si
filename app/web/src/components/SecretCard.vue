<template>
  <div
    :class="
      clsx(
        'flex flex-row flex-none items-center overflow-hidden text-shade-100 dark:text-shade-0',
        detailedListItem
          ? 'border-b border-neutral-200 dark:border-neutral-500'
          : 'border rounded h-[90px]',
        !detailedListItem && secret.isUsable
          ? 'cursor-pointer border-neutral-500 dark:hover:bg-action-700 hover:bg-action-100 dark:hover:outline-action-300 hover:outline-action-500 hover:outline -outline-offset-1'
          : 'cursor-default border-destructive-600',
      )
    "
  >
    <div class="flex flex-col gap-1 px-sm py-xs grow capsize overflow-hidden">
      <div
        :class="
          clsx(
            'text-md font-bold leading-tight',
            detailedListItem ? 'break-words' : 'truncate',
            secret.isUsable
              ? 'text-neutral-500 dark:text-neutral-300'
              : 'text-destructive-500 font-bold',
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
            'text-xs',
            !detailedListItem && 'truncate',
            secret.isUsable
              ? 'text-neutral-500 dark:text-neutral-300'
              : 'text-destructive-500 font-bold',
          )
        "
      >
        <template v-if="secret.isUsable">
          Created:
          <Timestamp
            :date="new Date(secret.createdInfo.timestamp)"
            :relative="!detailedListItem"
            :size="detailedListItem ? 'extended' : 'normal'"
          />
          by
          {{ secret.createdInfo.actor.label }}
        </template>
        <template v-else>
          Created in another workspace. Edit secret to be able to use it.
        </template>
      </div>
      <div class="grow flex flex-row items-center">
        <div
          :class="
            clsx(
              'italic text-xs text-neutral-400',
              !detailedListItem && 'line-clamp-2',
            )
          "
        >
          Connected Components: {{ secret.connectedComponents.length }}
        </div>
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
        iconIdleTone="neutral"
        iconTone="action"
        tooltip="Edit"
        @click="emit('edit')"
      />
      <IconButton
        :disabled="secret.connectedComponents.length > 0"
        icon="trash"
        iconIdleTone="neutral"
        iconTone="destructive"
        tooltip="Delete"
        @click="deleteSecret"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Timestamp } from "@si/vue-lib/design-system";
import { PropType } from "vue";
import clsx from "clsx";
import { Secret, useSecretsStore } from "@/store/secrets.store";
import IconButton from "./IconButton.vue";

const props = defineProps({
  secret: { type: Object as PropType<Secret>, required: true },
  detailedListItem: { type: Boolean },
});

const secretsStore = useSecretsStore();

const deleteSecret = async () => {
  if (!props.secret || !props.secret.id) return;

  await secretsStore.DELETE_SECRET(props.secret.id);
};

const emit = defineEmits<{
  (e: "edit"): void;
}>();
</script>
