<template>
  <div
    class="flex flex-row flex-1 basis-1/2 items-center min-w-0 h-full justify-end"
  >
    <Collaborators />
    <Notifications />

    <template v-if="featureFlagsStore.SQLITE_TOOLS">
      <NavbarButton icon="odin" size="sm">
        <template #dropdownContent>
          <DropdownMenuItem
            v-if="
              changeSetsStore.selectedWorkspacePk &&
              changeSetsStore.selectedChangeSetId
            "
            icon="niflheim"
            label="Re-do Cold Start"
            @click="
              heimdall.niflheim(
                changeSetsStore.selectedWorkspacePk,
                changeSetsStore.selectedChangeSetId,
                true,
              )
            "
          />
          <DropdownMenuItem
            icon="mjolnir"
            label="Throw Hammer"
            @click="() => modalRef.open()"
          />
          <DropdownMenuItem
            v-if="changeSetsStore.selectedChangeSetId"
            icon="odin"
            label="Log Sqlite"
            @click="
              () =>
                changeSetsStore.selectedChangeSetId &&
                heimdall.odin(changeSetsStore.selectedChangeSetId)
            "
          />
          <DropdownMenuItem
            icon="trash"
            label="Bobby Drop Tables"
            @click="() => heimdall.bobby()"
          />
        </template>
      </NavbarButton>
    </template>

    <template v-if="!collapse">
      <NavbarButton
        tooltipText="Documentation"
        icon="question-circle"
        externalLinkTo="https://docs.systeminit.com/"
      />

      <NavbarButton
        tooltipText="Discord Community"
        icon="logo-discord"
        externalLinkTo="https://discord.gg/system-init"
      />

      <WorkspaceSettingsMenu />
    </template>

    <ProfileButton :showTopLevelMenuItems="collapse" />

    <Modal ref="modalRef" title="Throw">
      <Stack>
        <VormInput v-model="entityKind" label="Entity Kind" type="text" />
        <VormInput v-model="entityId" label="ID" type="text" />
        <VButton
          label="Mjolnir!"
          tone="action"
          variant="soft"
          @click="hammer"
        />
      </Stack>
    </Modal>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
  DropdownMenuItem,
  VormInput,
  VButton,
  Modal,
  Stack,
} from "@si/vue-lib/design-system";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import * as heimdall from "@/store/realtime/heimdall";
import { useChangeSetsStore } from "@/store/change_sets.store";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import Notifications from "./Notifications.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";

const featureFlagsStore = useFeatureFlagsStore();
const changeSetsStore = useChangeSetsStore();
const modalRef = ref();
const entityId = ref("");
const entityKind = ref("");

const windowWidth = ref(window.innerWidth);
const collapse = computed(() => windowWidth.value < 1200);

const windowResizeHandler = () => {
  windowWidth.value = window.innerWidth;
};

const hammer = () => {
  if (
    changeSetsStore.selectedWorkspacePk &&
    changeSetsStore.selectedChangeSetId
  ) {
    heimdall.mjolnir(
      changeSetsStore.selectedWorkspacePk,
      changeSetsStore.selectedChangeSetId,
      entityKind.value,
      entityId.value,
    );
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
