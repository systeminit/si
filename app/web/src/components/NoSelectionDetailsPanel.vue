<template>
  <ScrollArea>
    <template #top>
      <SidebarSubpanelTitle
        :label="
          changeSetStore.headSelected
            ? 'HEAD'
            : changeSetStore.selectedChangeSet?.name
        "
        :icon="changeSetStore.headSelected ? 'git-branch' : 'git-branch'"
      >
        <div
          v-if="userIsApprover && pendingApprovalCount > 0"
          :class="
            clsx(
              'text-sm font-bold hover:underline cursor-pointer',
              themeClasses('text-action-500', 'text-action-400'),
            )
          "
          @click="openPendingApprovalsModal"
        >
          {{ pendingApprovalCount }} Pending Approval{{
            pendingApprovalCount > 1 ? "s" : ""
          }}
        </div>
      </SidebarSubpanelTitle>

      <div
        v-if="!changeSetStore.headSelected"
        :class="
          clsx(
            'flex flex-row items-center justify-center text-neutral-400 gap-xs border-b shrink-0',
            themeClasses('border-neutral-200', 'border-neutral-600'),
          )
        "
      >
        <ApplyChangeSetButton class="grow" />
      </div>
    </template>

    <div class="absolute inset-0">
      <TabGroup
        startSelectedTabSlug="changes"
        rememberSelectedTabKey="no-selection-details-panel"
        trackingSlug="no-selection-details-panel"
      >
        <TabGroupItem label="Changes" slug="changes">
          <ChangesPanelProposed />
        </TabGroupItem>
        <TabGroupItem label="History" slug="history">
          <ChangesPanelHistory />
        </TabGroupItem>
        <TabGroupItem label="Secrets" slug="secrets">
          <SecretsPanel />
        </TabGroupItem>
      </TabGroup>
    </div>
    <ApprovalPendingModal
      v-if="pendingApprovalCount > 0"
      ref="pendingApprovalModalRef"
    />
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  TabGroup,
  TabGroupItem,
  themeClasses,
  ScrollArea,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { computed, ref } from "vue";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import SecretsPanel from "./SecretsPanel.vue";
import ChangesPanelProposed from "./ChangesPanelProposed.vue";
import ChangesPanelHistory from "./ChangesPanelHistory.vue";
import ApprovalPendingModal from "./ApprovalPendingModal.vue";

const changeSetStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();

const pendingApprovalModalRef = ref<InstanceType<
  typeof ApprovalPendingModal
> | null>(null);

const userIsApprover = computed(() => changeSetStore.currentUserIsApprover);
const pendingApprovalCount = computed(
  () => changeSetStore.changeSetsNeedingApproval.length,
);

const openPendingApprovalsModal = () => {
  pendingApprovalModalRef.value?.open();
};
</script>
