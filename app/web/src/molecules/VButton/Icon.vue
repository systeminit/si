<template>
  <div aria-hidden="true" :class="iconClasses">
    <GitBranchIcon v-if="icon == 'git-branch'" />
    <GitCommitIcon v-else-if="icon == 'git-commit'" />
    <GitMergeIcon v-else-if="icon == 'git-merge'" />
    <PlusIcon v-else-if="icon == 'plus'" />
    <TrashIcon v-else-if="icon == 'trash'" />
    <XIcon v-else-if="icon == 'x'" />
  </div>
</template>

<script setup lang="ts">
import GitBranchIcon from "@/atoms/CustomIcons/GitBranchIcon.vue";
import GitCommitIcon from "@/atoms/CustomIcons/GitCommitIcon.vue";
import GitMergeIcon from "@/atoms/CustomIcons/GitMergeIcon.vue";
import { XIcon, PlusIcon, TrashIcon } from "@heroicons/vue/solid";

export type IconName =
  | "git-branch"
  | "git-commit"
  | "git-merge"
  | "plus"
  | "plus-square"
  | "trash"
  | "x";

defineProps<{
  icon: IconName;
  iconClasses: string[];
}>();
</script>
