<template>
  <div aria-hidden="true">
    <GitBranchIcon v-if="icon === 'git-branch'" :class="iconClasses" />
    <GitCommitIcon v-else-if="icon === 'git-commit'" :class="iconClasses" />
    <GitMergeIcon v-else-if="icon === 'git-merge'" :class="iconClasses" />
    <PlusIcon v-else-if="icon === 'plus'" :class="iconClasses" />
    <MinusIcon v-else-if="icon === 'minus'" :class="iconClasses" />
    <VueFeather
      v-else-if="icon === 'plus-square'"
      :class="iconClasses"
      type="plus-square"
    />
    <TrashIcon v-else-if="icon === 'trash'" :class="iconClasses" />
    <XIcon v-else-if="icon === 'x'" :class="iconClasses" />
    <ChevronDownIcon v-else-if="icon === 'chevron-down'" :class="iconClasses" />
    <SaveIcon v-else-if="icon === 'save'" :class="iconClasses" />
    <PlayIcon v-else-if="icon === 'play'" :class="iconClasses" />
  </div>
</template>

<script lang="ts" setup>
import {
  ChevronDownIcon,
  MinusIcon,
  PlusIcon,
  SaveIcon,
  TrashIcon,
  XIcon,
  PlayIcon,
} from "@heroicons/vue/solid";
import VueFeather from "vue-feather";
import GitBranchIcon from "@/atoms/CustomIcons/GitBranchIcon.vue";
import GitCommitIcon from "@/atoms/CustomIcons/GitCommitIcon.vue";
import GitMergeIcon from "@/atoms/CustomIcons/GitMergeIcon.vue";

export type IconName =
  | "git-branch"
  | "git-commit"
  | "git-merge"
  | "plus"
  | "minus"
  | "plus-square"
  | "trash"
  | "chevron-down"
  | "save"
  | "x"
  | "play";

defineProps<{
  icon: IconName;
  iconClasses?: string[];
}>();
</script>
