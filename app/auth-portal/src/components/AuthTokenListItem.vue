<template>
  <tr
    class="children:px-md children:py-sm children:truncate text-sm font-medium text-gray-800 dark:text-gray-200"
  >
    <td class="">
      <div
        class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
      >
        {{ authToken.name }}
      </div>
    </td>
    <!-- TODO show user of token if it's not current user--right now only owner can create -->
    <td class="normal-case">{{ authToken.createdAt }}</td>
    <td class="normal-case">{{ authToken.expiresAt }}</td>
    <td class="normal-case">
      <ErrorMessage :asyncState="revoke" />
      <VButton
        v-if="workspace.role === 'OWNER'"
        icon="trash"
        :loading="revoke.isLoading.value"
        class="cursor-pointer hover:text-destructive-500"
        @click="revoke.execute()"
      />
    </td>
  </tr>
</template>

<script lang="ts" setup>
import { ErrorMessage, VButton } from "@si/vue-lib/design-system";
import { apiData } from "@si/vue-lib/pinia";
import { useAsyncState } from "@vueuse/core";
import { Workspace } from "@/store/workspaces.store";
import { AuthToken, useAuthTokensApi } from "@/store/authTokens.store";

const api = useAuthTokensApi();

const props = defineProps<{
  authToken: AuthToken;
  workspace: Readonly<Workspace>;
}>();

const emit = defineEmits<{
  (e: "revoked"): void;
  (e: "renamed", newName: string): void;
}>();

/** Action to revoke token */
const revoke = useAsyncState(
  async () => {
    const { workspace, authToken } = props;
    await apiData(api.REVOKE_AUTH_TOKEN(workspace.id, authToken.id));
    emit("revoked");
  },
  undefined,
  { immediate: false },
);

// /** Action to rename token */
// const rename = useAsyncState(
//   async () => {
//     const { workspace, authToken } = props;
//     await apiData(
//       api.RENAME_AUTH_TOKEN(workspace.id, authToken.id, name.value),
//     );
//     emit("renamed", name.value);
//   },
//   undefined,
//   { immediate: false },
// );
// async function renameAuthToken(tokenId: AuthTokenId, name: string) {
// }
// /** Name of token to create */
// const name = ref(authToken.name);
</script>
