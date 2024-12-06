<template>
  <ScrollArea>
    <template #top>
      <SidebarSubpanelTitle
        :label="
          changeSetsStore.headSelected
            ? 'HEAD'
            : changeSetsStore.selectedChangeSet?.name
        "
        :icon="changeSetsStore.headSelected ? 'git-branch' : 'git-branch'"
      >
        <template v-if="!changeSetsStore.headSelected">
          <DetailsPanelMenuIcon
            :selected="dropdownMenuRef?.isOpen"
            @click="openMenu"
          />
          <DropdownMenu ref="dropdownMenuRef">
            <DropdownMenuItem
              label="Rename"
              icon="cursor"
              @select="openRenameModal"
            />
          </DropdownMenu>
          <Modal
            ref="renameModalRef"
            type="save"
            size="sm"
            saveLabel="Rename"
            title="Rename Change Set"
            @save="updateName"
          >
            <VormInput
              ref="labelRef"
              v-model="changeSetName"
              required
              label="Change Set Name"
              @enterPressed="updateName"
            />
          </Modal>
        </template>
      </SidebarSubpanelTitle>
      <div
        v-if="
          featureFlagsStore.REBAC && userIsApprover && pendingApprovalCount > 0
        "
        :class="
          clsx(
            'group/tree',
            'px-xs py-2xs flex flex-row cursor-pointer font-bold border border-transparent',
            changeSetsStore.headSelected &&
              themeClasses('border-b-neutral-200', 'border-b-neutral-600'),
            themeClasses(
              'hover:text-action-500  hover:border-action-500',
              'hover:text-action-300 bg-neutral-700  hover:border-action-300',
            ),
          )
        "
        @click="openPendingApprovalsModal"
      >
        <div class="text-sm grow group-hover/tree:underline">
          Pending Approval{{ pendingApprovalCount > 1 ? "s" : "" }}
        </div>
        <PillCounter :count="pendingApprovalCount" showHoverInsideTreeNode />
      </div>
      <div
        v-if="!changeSetsStore.headSelected"
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
  PillCounter,
  DropdownMenu,
  DropdownMenuItem,
  Modal,
  VormInput,
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
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const changeSetsStore = useChangeSetsStore();
const featureFlagsStore = useFeatureFlagsStore();

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenu>>();
const renameModalRef = ref<InstanceType<typeof Modal>>();

const pendingApprovalModalRef = ref<InstanceType<
  typeof ApprovalPendingModal
> | null>(null);

const userIsApprover = computed(() => changeSetsStore.currentUserIsApprover);
const pendingApprovalCount = computed(
  () => changeSetsStore.changeSetsNeedingApproval.length,
);

const openPendingApprovalsModal = () => {
  pendingApprovalModalRef.value?.open();
};

const openMenu = (e: MouseEvent) => {
  dropdownMenuRef.value?.open(e);
};

const openRenameModal = () => {
  if (!changeSetsStore.selectedChangeSet) return;
  changeSetName.value = changeSetsStore.selectedChangeSet.name;
  renameModalRef.value?.open();
};

const changeSetName = ref("");

const updateName = () => {
  if (!changeSetsStore.selectedChangeSetId) return;
  changeSetsStore.RENAME_CHANGE_SET(
    changeSetsStore.selectedChangeSetId,
    changeSetName.value,
  );
  renameModalRef.value?.close();
};
</script>
