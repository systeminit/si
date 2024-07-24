<template>
  <div class="overflow-hidden">
    <template v-if="featureFlagsStore.ADMIN_PAGE">
      <div class="mt-sm pb-md">
        <div>
          Use this page to create a production workspace for an invited user.
        </div>
      </div>

      <Stack>
        <ErrorMessage :requestStatus="createWorkspaceReqStatus" />
        <VormInput
          v-model="workspace.userEmail"
          label="Email"
          placeholder="The email of the user to invite"
          required
          :maxLength="500"
        />

        <VButton
          iconRight="chevron--right"
          :requestStatus="createWorkspaceReqStatus"
          loadingText="Creating..."
          tone="action"
          variant="solid"
          @click="createWorkspace()"
        >
          Create Workspace
        </VButton>
      </Stack>
    </template>
    <template v-else> </template>
  </div>
</template>

<script setup lang="ts">
import * as _ from "lodash-es";
import { onMounted, reactive } from "vue";
import {
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const authStore = useAuthStore();
const router = useRouter();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

const invitedUser = {
  userEmail: "",
};
const workspace = reactive(_.cloneDeep(invitedUser));

const createWorkspaceReqStatus = workspacesStore.getRequestStatus(
  "SETUP_PRODUCTION_WORKSPACE",
);

const createWorkspace = async () => {
  await workspacesStore.SETUP_PRODUCTION_WORKSPACE(workspace.userEmail);
};

onMounted(async () => {
  if (!authStore.userIsLoggedIn) return;

  if (
    !authStore.user?.email?.includes("@systeminit.com") &&
    !featureFlagsStore.ADMIN_PAGE
  ) {
    await router.push({
      name: "workspaces",
    });
  }
});
</script>
