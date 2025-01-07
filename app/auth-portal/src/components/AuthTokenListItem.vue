<template>
  <tr
    :class="
      clsx(
        themeClasses(
          'odd:bg-neutral-200 even:bg-neutral-100',
          'odd:bg-neutral-700 even:bg-neutral-800',
        ),
      )
    "
    class="children:px-sm children:py-xs children:truncate text-sm font-medium text-gray-800 dark:text-gray-200"
  >
    <td>
      <div
        class="xl:max-w-[800px] lg:max-w-[60vw] md:max-w-[50vw] sm:max-w-[40vw] max-w-[150px] truncate"
      >
        {{ authToken.name }}
      </div>
    </td>
    <!-- TODO show user of token if it's not current user--right now only owner can create -->
    <td>{{ createdAt }}</td>
    <td>{{ expiresAt }}</td>
    <!--td class="text-center">
      <ErrorMessage :asyncState="revoke" />
      <VButton
        v-if="workspace.role === 'OWNER'"
        :loading="revoke.isLoading.value"
        class="cursor-pointer"
        icon="trash"
        loadingText=""
        size="2xs"
        variant="transparent"
        @click="revoke.execute()"
      />
    </td-->
  </tr>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import { computed } from "vue";
import { Workspace } from "@/store/workspaces.store";
import { AuthToken } from "@/store/authTokens.store";

// const api = useAuthTokensApi();

const props = defineProps<{
  authToken: AuthToken;
  workspace: Readonly<Workspace>;
}>();

const emit = defineEmits<{
  (e: "revoked"): void;
  (e: "renamed", newName: string): void;
}>();

/** Action to revoke token */
// const revoke = useAsyncState(
//   async () => {
//     const { workspace, authToken } = props;
//     await apiData(api.REVOKE_AUTH_TOKEN(workspace.id, authToken.id));
//     emit("revoked");
//   },
//   undefined,
//   { immediate: false },
// );

const createdAt = computed(() =>
  new Date(props.authToken.createdAt).toLocaleString(),
);

const expiresAt = computed(() =>
  props.authToken.expiresAt
    ? new Date(props.authToken.expiresAt).toLocaleString()
    : undefined,
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
