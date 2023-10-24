<template>
  <ScrollArea>
    <template #top>
      <SidebarSubpanelTitle
        :label="
          changeSetStore.headSelected
            ? 'Workspace Activity'
            : 'Change Set Details'
        "
        :icon="changeSetStore.headSelected ? 'git-branch' : 'git-branch'"
      />

      <div
        v-if="
          !changeSetStore.headSelected && componentsStore.allComponents.length
        "
        :class="
          clsx(
            'flex flex-row items-center justify-center text-neutral-400 gap-2 p-xs border-b shrink-0',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <ApplyChangeSetButton class="grow" />
        <strong
          class="text-action-300 bg-action-100 text-lg rounded-2xl px-3 border border-action-300"
        >
          {{
            1 +
            diffs.length +
            (changeSetStore.selectedChangeSet?.actions?.length ?? 0)
          }}
        </strong>
      </div>
    </template>

    <template v-if="componentsStore.allComponents.length === 0">
      <div class="flex flex-col items-center text-neutral-400 pt-lg">
        <EmptyStateIcon name="no-assets" class="mt-3" />
        <span class="text-xl dark:text-neutral-300">Your model is empty</span>
        <div class="capsize px-xs py-md italic text-sm text-center">
          Drag some assets onto the diagram
        </div>
      </div>
    </template>

    <template v-else>
      <div class="absolute inset-0">
        <!-- <ApplyHistory  /> -->
        <TabGroup
          rememberSelectedTabKey="proposed_right"
          trackingSlug="actions_applied"
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
              <div class="flex flex-col">
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

            <div
              v-for="diff in diffs"
              :key="diff.componentId"
              :class="
                clsx(
                  'flex flex-row gap-xs items-center text-sm p-xs border-b',
                  themeClasses('border-neutral-200', 'border-neutral-600'),
                )
              "
            >
              <StatusIndicatorIcon
                type="change"
                :status="diff.status"
                tone="shade"
              />
              <div class="flex flex-col">
                <div class="">
                  <span v-if="diff.status === 'added'">Added</span>
                  <span v-if="diff.status === 'deleted'">Removed</span>
                  <span v-if="diff.status === 'modified'">Modified</span>
                  {{
                    componentsStore.componentsById[diff.componentId]?.schemaName
                  }}
                </div>
                <div class="text-neutral-400 truncate">
                  {{
                    componentsStore.componentsById[diff.componentId]
                      ?.displayName
                  }}
                </div>
              </div>
            </div>

            <div
              v-for="action in actionsStore.proposedActions"
              :key="action.actionInstanceId"
              :class="
                clsx(
                  'border-b',
                  themeClasses('border-neutral-200', 'border-neutral-600'),
                )
              "
            >
              <ActionSprite
                :action="action"
                @remove="actionsStore.REMOVE_ACTION(action.actionInstanceId)"
              />
            </div>
            <div
              v-if="!actionsStore.proposedActions?.length"
              class="p-4 italic !delay-0 !duration-0 hidden first:block"
            >
              <div class="pb-sm">No actions were chosen at this time.</div>
            </div>
          </TabGroupItem>

          <TabGroupItem label="Applied Changes" slug="actions_applied">
            <ApplyHistory />
          </TabGroupItem>
        </TabGroup>
      </div>
    </template>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed } from "vue";
import {
  TabGroup,
  TabGroupItem,
  themeClasses,
  Icon,
  ScrollArea,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useComponentsStore } from "@/store/components.store";
import { useChangeSetsStore } from "@/store/change_sets.store";
import ActionSprite from "@/components/ActionSprite.vue";

import { useActionsStore } from "@/store/actions.store";
import ApplyHistory from "./ApplyHistory.vue";

import StatusIndicatorIcon from "./StatusIndicatorIcon.vue";
import EmptyStateIcon from "./EmptyStateIcon.vue";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";

const changeSetStore = useChangeSetsStore();
const actionsStore = useActionsStore();

const diffs = computed(() => {
  const arr = Object.values(componentsStore.componentsById)
    .filter((c) => c.changeStatus !== "unmodified")
    .map((c) => {
      let updatedAt = c.updatedInfo.timestamp;
      if (c.changeStatus === "added") {
        updatedAt = c.createdInfo.timestamp;
      } else if (c.changeStatus === "deleted" && c.deletedInfo) {
        updatedAt = c.deletedInfo.timestamp;
      }

      return {
        componentId: c.id,
        status: c.changeStatus,
        updatedAt,
      };
    });
  arr.sort(
    (a, b) => new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime(),
  );
  return arr;
});

const componentsStore = useComponentsStore();
</script>
