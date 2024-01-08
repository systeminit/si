<template>
  <TabGroup
    rememberSelectedTabKey="proposed_right"
    trackingSlug="actions_applied"
    minimal
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
          <div class="text-neutral-400 truncate">
            {{
              changeSetStore.headSelected
                ? "head"
                : changeSetStore.selectedChangeSet?.name
            }}
          </div>
        </div>
      </div>

      <ChangeCard v-for="diff in diffs" :key="diff.componentId" :diff="diff" />

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
        <ActionSprite
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
import { computed } from "vue";
import { useActionsStore } from "@/store/actions.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useComponentsStore } from "@/store/components.store";
import ApplyHistory from "./ApplyHistory.vue";
import ActionSprite from "./ActionSprite.vue";
import ChangeCard from "./ChangeCard.vue";

const changeSetStore = useChangeSetsStore();
const actionsStore = useActionsStore();
const componentsStore = useComponentsStore();

const diffs = computed(() => {
  const arr = Object.values(componentsStore.componentsById)
    .filter((c) => c.changeStatus !== "unmodified")
    .map((c) => {
      let updatedAt = c.updatedInfo.timestamp;
      let actor = c.updatedInfo.actor.email || c.updatedInfo.actor.label;
      if (c.changeStatus === "added") {
        updatedAt = c.createdInfo.timestamp;
        actor = c.createdInfo.actor.email || c.createdInfo.actor.label;
      } else if (c.changeStatus === "deleted" && c.deletedInfo) {
        updatedAt = c.deletedInfo.timestamp;
        actor = c.deletedInfo.actor.email || c.deletedInfo.actor.label;
      }

      return {
        componentId: c.id,
        status: c.changeStatus,
        updatedAt,
        actor,
      };
    });
  arr.sort(
    (a, b) => new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
  );
  return arr;
});
</script>
