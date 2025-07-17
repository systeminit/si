<template>
  <!-- Hover styles - hover:bg-action-200 dark:hover:bg-action-500 cursor-pointer -->
  <div
    class="flex flex-row items-center gap-xs p-xs pl-sm overflow-hidden flex-none"
  >
    <UserIcon
      :user="user"
      changeSetStarSide
      :hideChangesetStar="hideChangeSetInfo"
      :hasHoverState="iconHasHoverState"
      @click="emit('iconClicked')"
    />

    <div class="flex flex-col min-w-0">
      <div class="w-full truncate leading-tight">
        {{ user.name }}
      </div>
      <div
        v-if="!hideChangeSetInfo"
        class="text-xs font-bold line-clamp-3 break-words"
      >
        {{ changeSetName }}
      </div>
      <div v-if="!hideStatus" class="text-xs italic line-clamp-3 break-words">
        {{ user.status }}
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed, inject, PropType } from "vue";
import UserIcon from "./UserIcon.vue";
import { UserInfo } from "./Collaborators.vue";
import { assertIsDefined, Context } from "../types";
import { useChangeSets } from "../logic_composables/change_set";

const ctx = inject<Context>("CONTEXT");
assertIsDefined(ctx);

const { openChangeSets } = useChangeSets(computed(() => ctx));

const changeSetName = computed(() => {
  return props.user.changeSet
    ? openChangeSets.value.find((c) => c.id === props.user.changeSet)?.name ||
        "Head"
    : "Head";
});

const props = defineProps({
  user: { type: Object as PropType<UserInfo>, required: true },
  hideChangeSetInfo: { type: Boolean },
  iconHasHoverState: { type: Boolean },
  hideStatus: { type: Boolean },
});

const emit = defineEmits<{
  (e: "iconClicked"): void;
}>();
</script>
