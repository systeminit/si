<template>
  <LoadStatus :requestStatus="workspaceStatus">
    <template #uninitialized>
      <div>You must log in to view workspace status.</div>
    </template>
    <template #success>
      <div class="overflow-hidden">
        <WorkspacePageHeader
          :title="`${workspace.displayName} > API Tokens`"
          subtitle="From here you can manage API tokens for your workspace. Enter the name and expiration of the token below and click the Generate API Token button"
        >
          <RouterLink
            :to="{ name: 'workspace-settings', params: { workspaceId } }"
          >
            <IconButton
              class="flex-none"
              icon="settings"
              iconIdleTone="shade"
              size="lg"
              tooltip="Settings"
              tooltipPlacement="top"
            />
          </RouterLink>
        </WorkspacePageHeader>

        <Stack>
          <ErrorMessage :asyncState="createAuthToken" />
          <form
            v-if="workspace.role === 'OWNER'"
            class="flex flex-row flex-wrap items-center justify-center gap-md"
          >
            <VormInput
              v-model="createAuthTokenName"
              inlineLabel
              label="Token Name"
              required
              placeholder="A name for your token."
              @keydown.enter.prevent="onFormSubmit"
            />
            <VormInput
              v-model="createAuthTokenExpiration"
              inlineLabel
              label="Expiration"
              required
              placeholder="48h, 1d, 1m, 1y, etc."
              type="time-string"
              :maxLength="99"
              @keydown.enter.prevent="onFormSubmit"
            />
            <VButton
              :disabled="validationState.isError"
              :loading="createAuthToken.isLoading.value"
              loadingText="Creating ..."
              tone="action"
              variant="solid"
              @click="onFormSubmit"
            >
              Generate API Token
            </VButton>
          </form>
          <ErrorMessage :asyncState="authTokens" />

          <AuthTokenList
            :workspace="workspace"
            :authTokens="activeTokens"
            active
            @renamed="renameToken"
            @revoked="revokeToken"
          />
          <AuthTokenList
            :workspace="workspace"
            :authTokens="inactiveTokens"
            @renamed="renameToken"
            @revoked="revokeToken"
          />
        </Stack>
      </div>

      <Modal ref="tokenDisplayModalRef" size="lg" title="Token Generated">
        <ErrorMessage
          v-if="tokenCopied"
          class="rounded-md text-md px-xs py-xs"
          icon="check-circle"
          tone="success"
          variant="block"
        >
          <b>Token copied!</b>
          We are only showing you the value of this token once. Store it
          somewhere secure, please.
        </ErrorMessage>
        <ErrorMessage
          v-else
          class="rounded-md text-md px-xs py-xs"
          icon="alert-circle"
          tone="info"
          variant="block"
        >
          We are only showing you the value of this token once. Store it
          somewhere secure, please.
        </ErrorMessage>
        <div class="flex flex-row items-center mt-sm gap-xs">
          <VormInput
            :modelValue="createAuthToken.state.value"
            class="flex-grow text-sm"
            disabled
            noLabel
            type="text"
          />
          <IconButton
            class="flex-none"
            icon="clipboard-copy"
            tooltip="Copy token to clipboard"
            tooltipPlacement="right"
            @click="copyToken"
          />
        </div>
      </Modal>
    </template>
  </LoadStatus>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, onMounted, onUnmounted, ref } from "vue";
import {
  VormInput,
  Stack,
  VButton,
  LoadStatus,
  ErrorMessage,
  Modal,
  IconButton,
  useValidatedInputGroup,
} from "@si/vue-lib/design-system";
import { useAsyncState } from "@vueuse/core";
import { apiData } from "@si/vue-lib/pinia";
import { useHead } from "@vueuse/head";
import { useWorkspacesStore, WorkspaceId } from "@/store/workspaces.store";
import WorkspacePageHeader from "@/components/WorkspacePageHeader.vue";
import { AuthToken, useAuthTokensApi } from "@/store/authTokens.store";
import AuthTokenList from "@/components/AuthTokenList.vue";

useHead({ title: "API Tokens" });

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

// This pokes the computed values to check if any tokens have expired every 5 seconds
const EXPIRATION_CHECK_INTERVAL = 5000;
const checkExpiration = ref();
const checkExpirationRecomputer = ref(0);

onMounted(() => {
  checkExpiration.value = setInterval(() => {
    checkExpirationRecomputer.value++;
    if (checkExpirationRecomputer.value > 100) {
      checkExpirationRecomputer.value = 0;
    }
  }, EXPIRATION_CHECK_INTERVAL);
});

onUnmounted(() => {
  clearInterval(checkExpiration.value);
});

// The list of all of the tokens to display
const listedTokens = computed(() => {
  return _.reverse(
    _.sortBy(_.values(authTokens.state.value), "createdAt"),
  ) as Array<AuthToken>;
});

// The list of all active tokens
const activeTokens = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  checkExpirationRecomputer.value;
  return listedTokens.value.filter((token) => {
    if (token.expiresAt === null || token.revokedAt) return false;
    const d = new Date(token.expiresAt as unknown as string);
    return d.getTime() > Date.now();
  });
});

// The list of all inactive tokens (expired or revoked)
const inactiveTokens = computed(() => {
  // eslint-disable-next-line @typescript-eslint/no-unused-expressions
  checkExpirationRecomputer.value;
  return listedTokens.value.filter(
    (token) => activeTokens.value.indexOf(token) === -1,
  );
});

/** Action to create auth token. Sets .state when done. */
const createAuthToken = useAsyncState(
  async () => {
    if (_.isEmpty(createAuthTokenName.value)) return;

    const { authToken, token } = await apiData(
      api.CREATE_AUTH_TOKEN(
        props.workspaceId,
        createAuthTokenName.value,
        createAuthTokenExpiration.value,
      ),
    );
    if (authTokens.state.value) {
      authTokens.state.value[authToken.id] = authToken;
    }

    tokenCopied.value = false;
    createAuthTokenName.value = "";
    createAuthTokenExpiration.value = "";
    validationMethods.resetAll();
    tokenDisplayModalRef.value?.open();

    return token;
  },
  undefined,
  { immediate: false, resetOnExecute: false },
);

/** Name of token to create */
const createAuthTokenName = ref("");
/** Expiration time of token to create */
const createAuthTokenExpiration = ref("");

/** Token modal */
const tokenDisplayModalRef = ref<InstanceType<typeof Modal> | null>(null);

const tokenCopied = ref(false);
async function copyToken() {
  if (createAuthToken.state.value) {
    await navigator.clipboard?.writeText(createAuthToken.state.value);
  }
  tokenCopied.value = true;
}

const { validationState, validationMethods } = useValidatedInputGroup();

const onFormSubmit = async () => {
  if (validationMethods.hasError()) return;

  await createAuthToken.execute();
};

// eslint-disable-next-line @typescript-eslint/no-unused-vars
const renameToken = (id: string, newName: string) => {
  // TODO(Wendy) - renaming tokens!
  // @renamed="(newName) => (authToken.name = newName)"
};

const revokeToken = (id: string) => {
  // TODO(Wendy) - revoking tokens!
  if (authTokens.state.value) {
    authTokens.state.value[id].revokedAt = new Date();
  }
  // @revoked="delete authTokens.state.value[authToken.id]"
};
</script>
