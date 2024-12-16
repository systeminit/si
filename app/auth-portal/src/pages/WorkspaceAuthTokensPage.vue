<template>
  <LoadStatus :requestStatus="workspaceStatus">
    <template #uninitialized>
      <div>You must log in to view workspace status.</div>
    </template>
    <template #success>
      <div class="overflow-hidden">
        <WorkspacePageHeader
          :title="`${workspace.displayName} > API Tokens`"
          subtitle="From here you can manage API tokens for your workspace. Enter the name and expiration of the API token below and click the Generate API Token button"
        >
          <RouterLink
            :to="{ name: 'workspace-settings', params: { workspaceId } }"
          >
            <Icon
              name="settings"
              tooltip="Settings"
              tooltipPlacement="top"
              size="lg"
              class="flex-none"
              iconTone="warning"
              iconIdleTone="shade"
              iconBgActiveTone="action"
            />
          </RouterLink>
        </WorkspacePageHeader>

        <Stack>
          <ErrorMessage :asyncState="createAuthToken" />
          <div class="flex flex-row flex-wrap items-end gap-xs">
            <VormInput
              v-model="createAuthTokenName"
              :disabled="workspace.role !== 'OWNER'"
              required
              label="Token Name"
              defaultValue="My API Token"
            />
            <VButton
              :disabled="workspace.role !== 'OWNER'"
              :loading="createAuthToken.isLoading.value"
              loadingText="Creating ..."
              iconRight="chevron--down"
              tone="action"
              variant="solid"
              @click="createAuthToken.execute()"
            >
              Create Automation Token
            </VButton>
          </div>
          <Stack
            v-if="createAuthToken.state.value"
            class="border-2 outline-2 rounded p-8"
          >
            <div>Token successfully created!</div>
            <div class="text-lg font-bold">
              This is the last time you will ever see this token value.
            </div>
            <VormInput
              :modelValue="createAuthToken.state.value"
              type="textarea"
              label="Your API Token"
              disabled
            />
            <div>
              Click the button below to copy it before navigating away or doing
              anything else!
            </div>
            <VButton
              icon="clipboard-copy"
              tone="action"
              label="Copy API Token To Clipboard"
              class="mt-xs"
              clickSuccess
              successText="Copied to clipboard!"
              @click="copyToken"
            />
          </Stack>
          <ErrorMessage :asyncState="authTokens" />
          <div v-if="authTokens.state.value" class="relative">
            <Stack>
              <table
                class="w-full divide-y divide-neutral-400 dark:divide-neutral-600 border-b border-neutral-400 dark:border-neutral-600"
              >
                <thead>
                  <tr
                    class="children:pb-xs children:px-md children:font-bold text-left text-xs uppercase"
                  >
                    <th scope="col">Name</th>
                    <th scope="col">Created</th>
                    <th scope="col">Expires</th>
                    <th scope="col">Revoke</th>
                  </tr>
                </thead>
                <tbody
                  class="divide-y divide-neutral-300 dark:divide-neutral-700"
                >
                  <AuthTokenListItem
                    v-for="authToken of Object.values(
                      authTokens.state.value,
                    ).reverse()"
                    :key="authToken.id"
                    :authToken="authToken"
                    :workspace="workspace"
                    @revoked="delete authTokens.state.value[authToken.id]"
                    @renamed="(newName) => (authToken.name = newName)"
                  />
                </tbody>
              </table>
            </Stack>
          </div>
        </Stack>
      </div>
    </template>
  </LoadStatus>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, ref } from "vue";
import {
  Icon,
  VormInput,
  Stack,
  VButton,
  LoadStatus,
  ErrorMessage,
} from "@si/vue-lib/design-system";
import { useAsyncState } from "@vueuse/core";
import { apiData } from "@si/vue-lib/pinia";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";
import WorkspacePageHeader from "@/components/WorkspacePageHeader.vue";
import { useAuthTokensApi } from "@/store/authTokens.store";
import AuthTokenListItem from "@/components/AuthTokenListItem.vue";

const workspacesStore = useWorkspacesStore();
const api = useAuthTokensApi();

const props = defineProps<{
  workspaceId: WorkspaceId;
}>();

// Fetch the workspace (by fetching all workspaces)
const workspaceStatus = workspacesStore.refreshWorkspaces();
const workspace = computed(
  () => workspacesStore.workspacesById[props.workspaceId],
);

/** The list of tokens */
const authTokens = useAsyncState(
  async () => {
    const { authTokens } = await apiData(
      api.FETCH_AUTH_TOKENS(props.workspaceId),
    );
    return _.keyBy(authTokens, "id");
  },
  undefined,
  { shallow: false },
);

/** Action to create auth token. Sets .state when done. */
const createAuthToken = useAsyncState(
  async () => {
    const { authToken, token } = await apiData(
      api.CREATE_AUTH_TOKEN(props.workspaceId, createAuthTokenName.value),
    );
    if (authTokens.state.value) {
      authTokens.state.value[authToken.id] = authToken;
    }
    return token;
  },
  undefined,
  { immediate: false, resetOnExecute: false },
);

/** Name of token to create */
const createAuthTokenName = ref("");

async function copyToken() {
  if (createAuthToken.state.value)
    await navigator.clipboard.writeText(createAuthToken.state.value);
}
</script>
