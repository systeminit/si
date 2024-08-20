<template>
  <div class="overflow-hidden">
    <template v-if="featureFlagsStore.ADMIN_PAGE">
      <h3 class="pb-md font-bold">
        Create production workspace for an invited user
      </h3>

      <p>Create by User Email</p>
      <Stack>
        <ErrorMessage :requestStatus="createWorkspaceReqStatus" />
        <VormInput
          v-model="workspace.userEmail"
          :maxLength="500"
          label="Email"
          placeholder="The email of the user to invite"
          required
        />

        <VButton
          :requestStatus="createWorkspaceReqStatus"
          iconRight="chevron--right"
          loadingText="Creating..."
          tone="action"
          variant="solid"
          @click="createWorkspace()"
        >
          Create Workspace For User Email Address
        </VButton>
      </Stack>
      <p class="mt-4">
        Create by User Id - this must be an ID that is in our auth-api database
      </p>
      <Stack>
        <ErrorMessage :requestStatus="createWorkspaceByUserIdReqStatus" />
        <VormInput
          v-model="workspaceById.userId"
          :maxLength="500"
          label="User ID"
          placeholder="The user id of the user to invite"
          required
        />

        <VButton
          :requestStatus="createWorkspaceByUserIdReqStatus"
          iconRight="chevron--right"
          loadingText="Creating..."
          tone="action"
          variant="solid"
          @click="createWorkspaceById()"
        >
          Create Workspace for Know SystemInit UserID
        </VButton>
      </Stack>
      <divider class="my-4" />
      <h3 class="pb-md font-bold">Manage Quarantined Accounts</h3>

      <Stack>
        <ErrorMessage :requestStatus="setUserQuarantineReqStatus" />
        <VormInput
          v-model="quarantineUserId"
          :maxLength="500"
          label="User ID"
          placeholder="The user id of the account to be managed"
          required
        />

        <div class="flex gap-2 px-2">
          <VButton
            :disabled="_.isEmpty(quarantineUserId)"
            :requestStatus="setUserQuarantineReqStatus"
            class="grow"
            icon="lock"
            tone="destructive"
            variant="solid"
            @click="setUserQuarantine(true)"
          >
            Quarantine Account
          </VButton>
          <VButton
            :disabled="_.isEmpty(quarantineUserId)"
            :requestStatus="setUserQuarantineReqStatus"
            class="grow"
            icon="lock-open"
            tone="success"
            variant="solid"
            @click="setUserQuarantine(false)"
          >
            Unquarantine Account
          </VButton>
        </div>
      </Stack>
    </template>
    <template v-else> Feature not Enabled for account </template>
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { onMounted, reactive, ref } from "vue";
import {
  VormInput,
  Stack,
  ErrorMessage,
  VButton,
  Divider,
} from "@si/vue-lib/design-system";
import { useRouter } from "vue-router";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { useFeatureFlagsStore } from "@/store/feature_flags.store";

const authStore = useAuthStore();
const router = useRouter();
const workspacesStore = useWorkspacesStore();
const featureFlagsStore = useFeatureFlagsStore();

const invitedUserByEmail = {
  userEmail: "",
};
const invitedUserById = {
  userId: "",
};
const workspace = reactive(_.cloneDeep(invitedUserByEmail));
const workspaceById = reactive(_.cloneDeep(invitedUserById));

const createWorkspaceReqStatus = workspacesStore.getRequestStatus(
  "SETUP_PRODUCTION_WORKSPACE",
);
const createWorkspace = async () => {
  await workspacesStore.SETUP_PRODUCTION_WORKSPACE(workspace.userEmail);
};

const createWorkspaceByUserIdReqStatus = workspacesStore.getRequestStatus(
  "SETUP_PRODUCTION_WORKSPACE_BY_USER_ID",
);
const createWorkspaceById = async () => {
  await workspacesStore.SETUP_PRODUCTION_WORKSPACE_BY_USER_ID(
    workspaceById.userId,
  );
};

// User quarantine state
const quarantineUserId = ref("");
const setUserQuarantineReqStatus = authStore.getRequestStatus(
  "SET_USER_QUARANTINE",
);
const setUserQuarantine = async (isQuarantined: boolean) => {
  await authStore.SET_USER_QUARANTINE(quarantineUserId.value, isQuarantined);
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
