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
              heimdall.muspelheim(changeSetsStore.selectedWorkspacePk, true)
            "
          />
          <DropdownMenuItem
            v-if="
              changeSetsStore.selectedWorkspacePk &&
              changeSetsStore.selectedChangeSetId
            "
            icon="refresh"
            label="Rebuild Index"
            @click="
              rebuild(
                changeSetsStore.selectedWorkspacePk,
                changeSetsStore.selectedChangeSetId,
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
          <DropdownMenuItem
            v-if="
              changeSetsStore.selectedWorkspacePk &&
              changeSetsStore.selectedChangeSetId
            "
            icon="trash"
            label="Ragnarok"
            @click="
              () =>
                heimdall.ragnarok(
                  changeSetsStore.selectedWorkspacePk!,
                  changeSetsStore.selectedChangeSetId!,
                )
            "
          />
        </template>
      </NavbarButton>
    </template>

    <template v-if="!collapse">
      <NavbarButton
        v-if="featureFlagsStore.ENABLE_NEW_EXPERIENCE"
        icon="grid"
        tooltipText="Use New UI"
        @click="toggleExperience"
      />

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

    <ApplyChangeSetButton v-if="useNewUI" />

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
import { useRouter, useRoute } from "vue-router";
import {
  DropdownMenuItem,
  VormInput,
  VButton,
  Modal,
  Stack,
} from "@si/vue-lib/design-system";
import { URLPattern, describePattern } from "@si/vue-lib";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import * as heimdall from "@/store/realtime/heimdall";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { sdfApiInstance } from "@/store/apis.web";
import { ChangeSetId } from "@/api/sdf/dal/change_set";
import { EntityKind } from "@/workers/types/entity_kind_types";
import ApplyChangeSetButton from "@/newhotness/ApplyChangeSetButton.vue";
import NavbarButton from "./NavbarButton.vue";
import Collaborators from "./Collaborators.vue";
import Notifications from "./Notifications.vue";
import WorkspaceSettingsMenu from "./WorkspaceSettingsMenu.vue";
import ProfileButton from "./ProfileButton.vue";

const featureFlagsStore = useFeatureFlagsStore();
const changeSetsStore = useChangeSetsStore();
const router = useRouter();
const route = useRoute();
const modalRef = ref();
const entityId = ref("");
const entityKind = ref("");

const windowWidth = ref(window.innerWidth);
const collapse = computed(() => windowWidth.value < 1200);

// Determine if we're in the new experience
const useNewUI = computed(() => {
  return route.name?.toString().startsWith("new-hotness");
});

// Simple toggle function that switches UI experience
const toggleExperience = () => {
  // Get current route information
  const workspacePk =
    route.params.workspacePk || changeSetsStore.selectedWorkspacePk;
  const changeSetId =
    route.params.changeSetId || changeSetsStore.selectedChangeSetId || "auto";
  const componentId = route.params.componentId;
  const funcRunId = route.params.funcRunId;

  if (!useNewUI.value) {
    // Currently in old UI, switch to new UI
    if (componentId) {
      // If we have a component ID, navigate to component detail view in new UI
      router.push({
        path: `/n/${workspacePk}/${changeSetId}/${componentId}/c`,
      });
    } else if (funcRunId) {
      // If we have a func run ID, navigate to func run detail view in new UI
      router.push({
        path: `/n/${workspacePk}/${changeSetId}/${funcRunId}/r`,
      });
    } else {
      // Otherwise navigate to the new UI dashboard
      router.push({
        path: `/n/${workspacePk}/${changeSetId}/h`,
      });
    }
  } else {
    // Currently in new UI, switch to old UI
    if (componentId) {
      // If we have a component ID, navigate to component view in old UI
      router.push({
        name: "workspace-compose",
        params: {
          workspacePk,
          changeSetId,
          componentId,
        },
      });
    } else {
      // Otherwise navigate to the old UI dashboard
      router.push({
        path: `/w/${workspacePk}/${changeSetId}/c`,
      });
    }
  }
};

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
  if (
    changeSetsStore.selectedWorkspacePk &&
    changeSetsStore.selectedChangeSetId
  ) {
    heimdall.mjolnir(
      changeSetsStore.selectedWorkspacePk,
      changeSetsStore.selectedChangeSetId,
      entityKind.value as EntityKind,
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
