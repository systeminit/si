<template>
  <div
    v-if="actionsStore.actions.length > 0"
    class="flex flex-col items-center"
  >
    <div class="w-52">
      <EmptyStateIcon name="actions" />
    </div>
    <div class="text-xl text-neutral-400 dark:text-neutral-300 mt-2">
      No Actions To Be Taken
    </div>
    <div class="text-sm px-xs pt-3 text-neutral-400 text-center italic">
      There are no <span class="font-bold">actions</span> to display for the
      selected asset(s)
    </div>
  </div>
  <ScrollArea v-else>
    <ActionRunnerCardV2
      v-for="action in actionsStore.actions"
      :key="action.id"
      :action="action"
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { ScrollArea } from "@si/vue-lib/design-system";
import { useActionsStore } from "@/store/actions.store";
import ActionRunnerCardV2 from "@/components/Actions/ActionRunnerCardV2.vue";
import EmptyStateIcon from "./EmptyStateIcon.vue";

// jobelenus:TODO I'm not sure if actionsStore.actions is right, but I know they are working on history right now

const actionsStore = useActionsStore();
</script>
