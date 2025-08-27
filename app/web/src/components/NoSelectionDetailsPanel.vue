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
        <TabGroupItem label="Secrets" slug="secrets"> </TabGroupItem>
      </TabGroup>
    </div>
  </ScrollArea>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import {
  TabGroup,
  TabGroupItem,
  themeClasses,
  ScrollArea,
  DropdownMenu,
  DropdownMenuItem,
  Modal,
  VormInput,
} from "@si/vue-lib/design-system";
import clsx from "clsx";
import { ref } from "vue";
import ApplyChangeSetButton from "@/components/ApplyChangeSetButton.vue";
import { useChangeSetsStore } from "@/store/change_sets.store";
import SidebarSubpanelTitle from "./SidebarSubpanelTitle.vue";
import ChangesPanelProposed from "./ChangesPanelProposed.vue";
import ChangesPanelHistory from "./ChangesPanelHistory.vue";
import DetailsPanelMenuIcon from "./DetailsPanelMenuIcon.vue";

const changeSetsStore = useChangeSetsStore();

const dropdownMenuRef = ref<InstanceType<typeof DropdownMenu>>();
const renameModalRef = ref<InstanceType<typeof Modal>>();

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
