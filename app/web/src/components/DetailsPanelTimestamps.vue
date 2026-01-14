<template>
  <div
    :class="
      clsx(
        'text-xs italic grow flex flex-col justify-between',
        themeClasses('text-neutral-500 ', 'text-neutral-300'),
        !noMargin && 'm-xs mt-0',
      )
    "
  >
    <div
      :class="
        clsx(
          changeStatus === 'added' && themeClasses('text-success-500', 'text-success-400'),
          'flex flex-row gap-2xs items-center',
        )
      "
    >
      <StatusIndicatorIcon type="change" status="added" size="xs" class="shrink-0" tone="inherit" />
      <div v-if="created" class="grow truncate">
        Created {{ formatters.timeAgo(created.timestamp) }} by
        {{ created.actor.label }}
      </div>
      <div v-else-if="changeStatus === 'added'">Created in this change set.</div>
      <div v-else>Created in a previous change set.</div>
    </div>
    <div
      v-if="
        modified &&
        (changeStatus === 'modified' || changeStatus === 'unmodified') &&
        created?.timestamp !== modified?.timestamp
      "
      :class="clsx(changeStatus === 'modified' && 'text-warning-500', 'flex flex-row gap-2xs items-center')"
    >
      <StatusIndicatorIcon type="change" status="modified" size="xs" class="shrink-0" tone="inherit" />
      <div v-if="modified" class="grow truncate">
        Modified {{ formatters.timeAgo(modified?.timestamp) }} by
        {{ modified?.actor.label }}
      </div>
    </div>
    <div
      v-if="changeStatus === 'deleted'"
      class="flex flex-row gap-2xs items-center text-destructive-500 dark:text-destructive-600"
    >
      <StatusIndicatorIcon type="change" status="deleted" size="xs" class="shrink-0" tone="inherit" />
      <div v-if="deleted" class="grow truncate">
        Deleted {{ formatters.timeAgo(deleted?.timestamp) }} by
        {{ deleted?.actor.label }}
      </div>
      <div v-else>Set for deletion</div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import clsx from "clsx";
import { formatters } from "@si/vue-lib";
import { themeClasses } from "@si/vue-lib/design-system";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp } from "@/api/sdf/dal/component";
import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";

defineProps({
  changeStatus: { type: String as PropType<ChangeStatus> },
  created: { type: Object as PropType<ActorAndTimestamp> },
  modified: { type: Object as PropType<ActorAndTimestamp> },
  deleted: { type: Object as PropType<ActorAndTimestamp> },
  noMargin: { type: Boolean },
});
</script>
