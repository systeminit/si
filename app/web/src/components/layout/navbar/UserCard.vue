<template>
  <div
    class="flex flex-row items-center gap-xs p-xs cursor-pointer overflow-hidden hover:bg-action-200 dark:hover:bg-action-500 flex-none"
  >
    <UserIcon :user="user" />

    <div class="flex flex-col min-w-0">
      <div class="w-full truncate leading-tight">
        {{ user.name }}
      </div>
      <div class="text-xs italic line-clamp-3 break-words">
        {{ user.status }}
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { PropType } from "vue";
import UserIcon from "./UserIcon.vue";
import { UserInfo } from "./Collaborators.vue";

defineProps({
  user: { type: Object as PropType<UserInfo>, required: true },
});
</script>
