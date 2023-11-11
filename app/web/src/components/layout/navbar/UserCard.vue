<template>
  <!-- Hover styles - hover:bg-action-200 dark:hover:bg-action-500 cursor-pointer -->
  <div
    class="flex flex-row items-center gap-xs p-xs pl-sm overflow-hidden flex-none"
  >
    <UserIcon
      :user="user"
      changeSetStarSide
      :hideChangesetStar="hideChangesetInfo"
    />

    <div class="flex flex-col min-w-0">
      <div class="w-full truncate leading-tight">
        {{ user.name }}
      </div>
      <div
        v-if="!hideChangesetInfo"
        class="text-xs font-bold line-clamp-3 break-words"
      >
        {{
          user.changeSetId
            ? changeSetsStore.changeSetsById[user.changeSetId]?.name || "Head"
            : "Head"
        }}
      </div>
      <div class="text-xs italic line-clamp-3 break-words">
        {{ user.idle ? "idle" : "active" }}
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { OnlineUserInfo } from "@/store/presence.store";
import UserIcon from "./UserIcon.vue";

const changeSetsStore = useChangeSetsStore();

defineProps({
  user: { type: Object as PropType<OnlineUserInfo>, required: true },
  hideChangesetInfo: { type: Boolean },
});
</script>
