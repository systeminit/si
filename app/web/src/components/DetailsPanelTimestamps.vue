<template>
  <div class="m-xs mt-0 text-xs italic text-neutral-300 grow">
    <div
      :class="
        clsx(
          changeStatus === 'added' && 'text-success-500',
          'flex flex-row gap-2xs items-center',
        )
      "
    >
      <StatusIndicatorIcon
        type="change"
        status="added"
        size="xs"
        class="shrink-0"
        tone="inherit"
      />
      <div class="grow truncate">
        {{ formatters.timeAgo(created.timestamp) }} by
        {{ created.actor.label }}
      </div>
    </div>
    <div
      v-if="
        modified &&
        (changeStatus === 'modified' || changeStatus === 'unmodified') &&
        created?.timestamp !== modified?.timestamp
      "
      :class="
        clsx(
          changeStatus === 'modified' && 'text-warning-500',
          'flex flex-row gap-2xs items-center',
        )
      "
    >
      <StatusIndicatorIcon
        type="change"
        status="modified"
        size="xs"
        class="shrink-0"
        tone="inherit"
      />
      <div class="grow truncate">
        {{ formatters.timeAgo(modified?.timestamp) }} by
        {{ modified?.actor.label }}
      </div>
    </div>
    <div
      v-if="changeStatus === 'deleted'"
      class="flex flex-row gap-2xs items-center text-destructive-500"
    >
      <StatusIndicatorIcon
        type="change"
        status="deleted"
        size="xs"
        class="shrink-0"
        tone="inherit"
      />
      <div class="grow truncate">
        {{ formatters.timeAgo(deleted?.timestamp) }} by
        {{ deleted?.actor.label }}
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import clsx from "clsx";
import { formatters } from "@si/vue-lib";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp } from "@/store/components.store";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

const props = defineProps({
  changeStatus: { type: String as PropType<ChangeStatus> },
  created: { type: Object as PropType<ActorAndTimestamp>, required: true },
  modified: { type: Object as PropType<ActorAndTimestamp> },
  deleted: { type: Object as PropType<ActorAndTimestamp> },
});
</script>
