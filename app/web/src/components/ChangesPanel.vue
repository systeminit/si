<template>
  <TabGroup
    rememberSelectedTabKey="proposed_right"
    trackingSlug="actions_applied"
    variant="minimal"
    marginTop="2xs"
  >
    <TabGroupItem
      v-if="!changeSetStore.headSelected"
      label="Proposed Changes"
      slug="actions_proposed"
    >
      <div
        :class="
          clsx(
            'flex flex-row gap-xs items-center text-sm p-xs border-b',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <Icon name="git-branch-plus" />
        <div class="flex flex-col overflow-hidden">
          <div class="">Created Change Set</div>
          <div class="text-neutral-500 dark:text-neutral-400 truncate">
            {{
              changeSetStore.headSelected
                ? "head"
                : changeSetStore.selectedChangeSet?.name
            }}
          </div>
        </div>
      </div>

      <template v-if="featureFlagsStore.IS_ACTIONS_V2">
        <div
          v-for="action in actionsStore.actionsV2"
          :key="action.id"
          :class="
            clsx(
              'border-b',
              themeClasses('border-neutral-200', 'border-neutral-600'),
            )
          "
        >
          <ActionV2Card
            :action="action"
            @remove="actionsStore.CANCEL(action.id)"
          />
        </div>
        <div
          v-if="!actionsStore.proposedActions.length"
          class="p-4 italic !delay-0 !duration-0 hidden first:block"
        >
          <div class="pb-sm">No actions were chosen at this time.</div>
        </div>
      </template>
      <template v-else>
        <div
          v-for="action in actionsStore.proposedActions"
          :key="action.id"
          :class="
            clsx(
              'border-b',
              themeClasses('border-neutral-200', 'border-neutral-600'),
            )
          "
        >
          <ActionCard
            :action="action"
            @remove="actionsStore.REMOVE_ACTION(action.id)"
          />
        </div>
        <div
          v-if="!actionsStore.proposedActions.length"
          class="p-4 italic !delay-0 !duration-0 hidden first:block"
        >
          <div class="pb-sm">No actions were chosen at this time.</div>
        </div>
      </template>
    </TabGroupItem>

    <TabGroupItem label="Applied Changes" slug="actions_applied">
      <ApplyHistory />
    </TabGroupItem>
  </TabGroup>
</template>

<script lang="ts" setup>
import {
  TabGroup,
  TabGroupItem,
  themeClasses,
  Icon,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { useActionsStore } from "@/store/actions.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import ApplyHistory from "./ApplyHistory.vue";
import ActionCard from "./ActionCard.vue";
import ActionV2Card from "./ActionV2Card.vue";

const changeSetStore = useChangeSetsStore();
const actionsStore = useActionsStore();
const featureFlagsStore = useFeatureFlagsStore();
</script>
