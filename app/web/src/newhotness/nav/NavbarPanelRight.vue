<template>
  <div class="flex flex-row flex-1 basis-1/2 items-center min-w-0 h-full justify-end">
    <template v-if="!invalidWorkspace">
      <Collaborators />
      <Notifications :changeSetsNeedingApproval="changeSetsNeedingApproval" />
    </template>

    <SQLiteToolsButton
      v-if="featureFlagsStore.SQLITE_TOOLS && !collapse"
      :changeSetId="changeSetId"
      :workspaceId="workspaceId"
    />

    <template v-if="!collapse">
      <NavbarButton tooltipText="Documentation" icon="question-circle" externalLinkTo="https://docs.systeminit.com/" />

      <NavbarButton
        tooltipText="Discord Community"
        icon="logo-discord"
        externalLinkTo="https://discord.gg/system-init"
      />

      <WorkspaceSettingsMenu />
    </template>

    <ProfileButton
      :showTopLevelMenuItems="collapse"
      :changeSetId="changeSetId"
      :workspaceId="workspaceId"
    />

    <ApplyChangeSetButton
      v-if="!invalidWorkspace"
      :squish="windowWidthReactive < 820"
    />
  </div>
</template>

<script lang="ts" setup>
import { computed } from "vue";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import ApplyChangeSetButton from "@/newhotness/ApplyChangeSetButton.vue";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import Notifications from "./Notifications.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";
import SQLiteToolsButton from "./SQLiteToolsButton.vue";
import { windowWidthReactive } from "../logic_composables/emitters";

const props = defineProps<{
  changeSetId: string;
  workspaceId: string;
  changeSetsNeedingApproval: ChangeSet[];
  invalidWorkspace?: boolean;
}>();

const featureFlagsStore = useFeatureFlagsStore();
const collapse = computed(() => windowWidthReactive.value < 1200);
</script>
