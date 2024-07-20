<template>
  <div v-if="user && user.emailVerified" class="overflow-hidden">
    <div
      class="pb-md flex flex-row gap-sm align-middle items-center justify-between"
    >
      <div>
        <div class="text-lg font-bold pb-sm">Your Workspaces</div>
        <div v-if="featureFlagsStore.CREATE_WORKSPACES">
          From here you can log into any of your workspaces.
        </div>
        <div v-else>
          From here you can log into your local dev instance. Eventually this
          will be where you can manage multiple workspaces, users,
          organizations, etc.
        </div>
      </div>
      <VButton
        v-if="featureFlagsStore.CREATE_WORKSPACES"
        label="Create Workspace"
        icon="plus"
        :linkTo="{ name: 'workspace-settings', params: { workspaceId: 'new' } }"
      />
    </div>
    <div
      class="mb-sm flex flex-col gap-sm p-sm border border-neutral-400 rounded-lg"
    >
      <div>Thank you for signing up!</div>
      <div>
        <span class="font-bold">System Initiative</span> is currently in Open
        Beta. In order to experience it, you will need to
        <a
          class="text-action-500 dark:text-action-300 font-bold hover:underline"
          href="https://github.com/systeminit/si/?tab=readme-ov-file#local-development-setup"
          target="_blank"
          >follow the instructions to get a local development environment set
          up</a
        >. Once the stack is up and running, you can click the button below to
        access your local development workspace. If you have questions or need
        help, join us on
        <a
          href="https://discord.gg/system-init"
          target="_blank"
          class="text-action-500 dark:text-action-300 font-bold hover:underline"
          >Discord</a
        >.
      </div>
    </div>
    <template v-if="loadWorkspacesReqStatus.isPending">
      <Icon name="loader" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isError">
      <ErrorMessage :requestStatus="loadWorkspacesReqStatus" />
    </template>
    <template v-else-if="loadWorkspacesReqStatus.isSuccess">
      <Stack>
        <WorkspaceLinkWidget
          v-for="workspace in sortedWorkspaces(workspaces)"
          :key="workspace.id"
          :workspaceId="workspace.id"
        />
      </Stack>
    </template>
  </div>
  <div v-else>
    You will not be able to use System Initiative until you verify your email.
  </div>
</template>

<script setup lang="ts">
import { computed, watch } from "vue";
import { Icon, Stack, ErrorMessage, VButton } from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore, Workspace } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";
import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

const workspaces = computed(() => workspacesStore.workspaces);
function sortedWorkspaces(workspaces: Workspace[]): Workspace[] {
  return workspaces.sort((a, b) => {
    // First, prioritize "SI" instanceEnvType
    if (a.instanceEnvType === "SI" && b.instanceEnvType !== "SI") {
      return -1;
    }
    if (a.instanceEnvType !== "SI" && b.instanceEnvType === "SI") {
      return 1;
    }

    // If both are "SI" or both are not "SI", sort by displayName
    return a.displayName.localeCompare(b.displayName);
  });
}

const user = computed(() => authStore.user);

useHead({ title: "Workspaces" });

const loadWorkspacesReqStatus =
  workspacesStore.getRequestStatus("LOAD_WORKSPACES");

function reloadWorkspaces() {
  if (import.meta.env.SSR) return;
  if (!authStore.userIsLoggedIn) return;

  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  workspacesStore.LOAD_WORKSPACES();
}
watch(() => authStore.userIsLoggedIn, reloadWorkspaces, { immediate: true });
</script>
