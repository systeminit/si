<template>
  <div class="m-xs mt-0 text-xs italic text-neutral-300">
    <Inline
      spacing="2xs"
      :class="clsx(changeStatus === 'added' && 'text-success-500')"
    >
      <Icon name="plus-circle" size="xs" />
      {{ formatters.timeAgo(created.timestamp) }} by
      {{ created.actor.label }}
    </Inline>
    <Inline
      v-if="
        modified &&
        (changeStatus === 'modified' || changeStatus === 'unmodified') &&
        created?.timestamp !== modified?.timestamp
      "
      spacing="2xs"
      :class="clsx(changeStatus === 'modified' && 'text-warning-500')"
    >
      <Icon name="tilde-circle" size="xs" />
      {{ formatters.timeAgo(modified?.timestamp) }} by
      {{ modified?.actor.label }}
    </Inline>
    <Inline
      v-if="changeStatus === 'deleted'"
      class="text-destructive-500"
      spacing="2xs"
    >
      <Icon name="minus-circle" size="xs" />
      {{ formatters.timeAgo(deleted?.timestamp) }} by
      {{ deleted?.actor.label }}
    </Inline>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import clsx from "clsx";
import Inline from "@/ui-lib/layout/Inline.vue";
import formatters from "@/ui-lib/helpers/formatting";
import { ChangeStatus } from "@/api/sdf/dal/change_set";
import { ActorAndTimestamp } from "@/store/components.store";
import Icon from "@/ui-lib/icons/Icon.vue";

const props = defineProps({
  changeStatus: { type: String as PropType<ChangeStatus> },
  created: { type: Object as PropType<ActorAndTimestamp>, required: true },
  modified: { type: Object as PropType<ActorAndTimestamp> },
  deleted: { type: Object as PropType<ActorAndTimestamp> },
});
</script>
