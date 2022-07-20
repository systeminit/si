<template>
  <div aria-hidden="true" :class="iconClasses">
    <GitBranchIcon v-if="icon == 'git-branch'" />
    <GitCommitIcon v-else-if="icon == 'git-commit'" />
    <GitMergeIcon v-else-if="icon == 'git-merge'" />
    <XIcon v-else-if="icon == 'x'" />
  </div>
</template>

<script setup lang="ts">
import GitBranchIcon from "@/atoms/CustomIcons/GitBranchIcon.vue";
import GitCommitIcon from "@/atoms/CustomIcons/GitCommitIcon.vue";
import GitMergeIcon from "@/atoms/CustomIcons/GitMergeIcon.vue";
import { XIcon } from "@heroicons/vue/solid";
import { ButtonIcon } from "./types";

defineProps<{
  icon: ButtonIcon;
  iconClasses: string[];
}>();
</script>
