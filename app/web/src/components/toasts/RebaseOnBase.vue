<template>
  <div class="flex flex-row items-center gap-xs">
    <Icon name="tools" class="action" />
    <div v-if="action === 'done'" class="flex flex-col text-sm text-center">
      Changes from HEAD have been applied to this change set.
    </div>
    <div v-if="action === 'prompt'" class="flex flex-col text-sm text-center">
      HEAD has changes you need to bring into this change set.
    </div>
    <div class="flex flex-row gap-sm items-center">
      <VButton
        v-if="action === 'prompt'"
        label="Rebase"
        tone="empty"
        :disabled="!changeSetStore.selectedChangeSetId"
        variant="solid"
        class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
        @click="rebase"
      ></VButton>
      <VButton
        class="grow text-action-300 dark:hover:text-white hover:text-black hover:bg-action-400 hover:underline"
        label="Dismiss"
        tone="empty"
        variant="solid"
        @click="$emit('close-toast')"
      ></VButton>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { Icon } from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";

const changeSetStore = useChangeSetsStore();

type Actions = "prompt" | "done";

defineProps<{
  action: Actions;
}>();

const rebase = () => {
  if (!changeSetStore.selectedChangeSetId) return;
  changeSetStore.REBASE_ON_BASE(changeSetStore.selectedChangeSetId);
};

const emit = defineEmits<{
  (e: "close-toast"): void;
}>();
</script>
