<template>
  <div class="flex flex-row flex-1 basis-1/2 items-center min-w-0 h-full justify-end">
    <template v-if="!invalidWorkspace">
      <Collaborators />
      <Notifications :changeSetsNeedingApproval="changeSetsNeedingApproval" />
    </template>

    <template v-if="featureFlagsStore.SQLITE_TOOLS">
      <NavbarButton icon="odin" size="sm">
        <template #dropdownContent>
          <DropdownMenuItem
            v-if="props.workspaceId && props.changeSetId"
            icon="niflheim"
            label="Re-do Cold Start"
            @click="heimdall.muspelheim(props.workspaceId, true)"
          />
          <DropdownMenuItem
            v-if="props.workspaceId && props.changeSetId"
            icon="refresh"
            label="Rebuild Index"
            @click="rebuild(props.workspaceId, props.changeSetId)"
          />
          <DropdownMenuItem icon="mjolnir" label="Throw Hammer" @click="() => modalRef.open()" />
          <DropdownMenuItem
            v-if="props.changeSetId"
            icon="odin"
            label="Log Sqlite"
            @click="() => props.changeSetId && heimdall.odin(props.changeSetId)"
          />
          <DropdownMenuItem icon="trash" label="Bobby Drop Tables" @click="() => heimdall.bobby()" />
          <DropdownMenuItem
            v-if="props.workspaceId && props.changeSetId"
            icon="trash"
            label="Ragnarok"
            @click="
              () => heimdall.ragnarok(props.workspaceId!, props.changeSetId!)
            "
          />
        </template>
      </NavbarButton>
    </template>

    <template v-if="!collapse">
      <NavbarButton tooltipText="Documentation" icon="question-circle" externalLinkTo="https://docs.systeminit.com/" />

      <NavbarButton
        tooltipText="Discord Community"
        icon="logo-discord"
        externalLinkTo="https://discord.gg/system-init"
      />

      <WorkspaceSettingsMenu />
    </template>

    <ProfileButton :showTopLevelMenuItems="collapse" />

    <ApplyChangeSetButton v-if="!invalidWorkspace" />

    <Modal ref="modalRef" title="Throw">
      <Stack>
        <VormInput v-model="entityKind" label="Entity Kind" type="text" />
        <VormInput v-model="entityId" label="ID" type="text" />
        <NewButton label="Mjolnir!" tone="action" @click="hammer" />
      </Stack>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { DropdownMenuItem, VormInput, Modal, Stack, NewButton } from "@si/vue-lib/design-system";
import { URLPattern, describePattern } from "@si/vue-lib";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import * as heimdall from "@/store/realtime/heimdall";
import { sdfApiInstance } from "@/store/apis.web";
import { ChangeSetId, ChangeSet } from "@/api/sdf/dal/change_set";
import { EntityKind } from "@/workers/types/entity_kind_types";
import ApplyChangeSetButton from "@/newhotness/ApplyChangeSetButton.vue";
import NavbarButton from "@/components/layout/navbar/NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import Notifications from "./Notifications.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";

const props = defineProps<{
  changeSetId: string;
  workspaceId: string;
  changeSetsNeedingApproval: ChangeSet[];
  invalidWorkspace?: boolean;
}>();

const featureFlagsStore = useFeatureFlagsStore();
const modalRef = ref();
const entityId = ref("");
const entityKind = ref("");

const windowWidth = ref(window.innerWidth);
const collapse = computed(() => windowWidth.value < 1200);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

const rebuild = (workspaceId: string, changeSetId: ChangeSetId) => {
  const pattern = [
    "v2",
    "workspaces",
    { workspaceId },
    "change-sets",
    { changeSetId },
    "index",
    "rebuild",
  ] as URLPattern;
  const [url] = describePattern(pattern);
  sdfApiInstance.post(url);
};

const hammer = () => {
  if (props.workspaceId && props.changeSetId) {
    heimdall.mjolnir(props.workspaceId, props.changeSetId, entityKind.value as EntityKind, entityId.value);
    modalRef.value.close();
  }
};

onMounted(() => {
  windowResizeHandler();
  window.addEventListener("resize", windowResizeHandler);
});
onBeforeUnmount(() => {
  window.removeEventListener("resize", windowResizeHandler);
});
</script>
