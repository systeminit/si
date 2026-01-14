<template>
  <div :class="clsx('relative h-8 w-8', hasHoverState && 'hover:z-50')">
    <div
      v-tooltip="tooltip"
      :class="
        clsx(
          'h-8 w-8 border-2 rounded-full cursor-pointer flex-none bg-shade-100 overflow-hidden',
          hasHoverState && 'hover:outline hover:outline-2',
          hasHoverState && forceDark
            ? 'hover:outline-action-300'
            : 'dark:hover:outline-action-300 hover:outline-action-500',
        )
      "
      :style="`border-color: ${color}`"
    >
      <!-- TODO(Wendy) - This should check for and pull the image of the user in question, not the current user's image! -->
      <img v-if="user.pictureUrl" class="rounded-full bg-shade-0" :src="user.pictureUrl" referrerpolicy="no-referrer" />
      <Icon v-else name="user-circle" size="full" class="text-shade-0" />
    </div>
    <div
      v-if="user.status === 'idle'"
      class="absolute top-0 w-full h-full z-90 opacity-60 bg-shade-100 rounded-full pointer-events-none"
    />
    <div
      v-if="
        !hideChangesetStar &&
        changeSetsStore.selectedChangeSetId &&
        changeSetsStore.selectedChangeSetId === user.changeSet
      "
      :class="
        clsx(
          'absolute w-full h-full z-100 flex flex-col items-center text-warning-300 pointer-events-none',
          changeSetStarSide ? 'top-[10px]' : 'top-0',
        )
      "
    >
      <Icon name="star" size="2xs" :class="changeSetStarSide ? 'translate-x-[-24px]' : 'translate-y-[-12px]'" />
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { PropType, computed } from "vue";
import clsx from "clsx";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { UserInfo } from "./Collaborators.vue";

const changeSetsStore = useChangeSetsStore();

const props = defineProps({
  tooltip: { type: Object },
  user: { type: Object as PropType<UserInfo>, required: true },
  changeSetStarSide: { type: Boolean },
  hideChangesetStar: { type: Boolean },
  hasHoverState: { type: Boolean },
  forceDark: { type: Boolean },
});

const color = computed(() => {
  if (props.user.color) return props.user.color;
  else return "black";
});
</script>
