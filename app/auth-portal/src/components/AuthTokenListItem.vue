<template>
  <tr
    :class="
      clsx(
        themeClasses(
          'odd:bg-neutral-200 even:bg-neutral-100',
          'odd:bg-neutral-700 even:bg-neutral-800',
        ),
        'children:p-xs children:truncate text-sm',
        active
          ? themeClasses('text-shade-100', 'text-shade-0')
          : themeClasses('text-neutral-700', 'text-neutral-300'),
      )
    "
  >
    <td>
      <TruncateWithTooltip>{{ token.name }}</TruncateWithTooltip>
    </td>
    <!-- TODO show user of token if it's not current user--right now only owner can create -->
    <td><Timestamp size="long" :date="createdAt" enableDetailTooltip /></td>
    <td :class="clsx(!active && expired && 'text-destructive-500 font-bold')">
      <Timestamp
        v-if="expiresAt"
        size="long"
        :date="expiresAt"
        enableDetailTooltip
      />
      <template v-else>Never</template>
    </td>
    <td v-if="workspace.role === 'OWNER'" class="text-center">
      <template v-if="active">
        <IconButton
          v-if="revoke.error.value"
          icon="alert-triangle"
          iconTone="destructive"
          class="w-min mx-auto"
          tooltip="Error. Token not revoked!"
          tooltipPlacement="top"
          @click="revoke.execute()"
        />
        <IconButton
          v-else
          :loading="revoke.isLoading.value"
          icon="trash"
          iconTone="destructive"
          iconIdleTone="shade"
          class="w-min mx-auto"
          tooltip="Revoke Token"
          tooltipPlacement="top"
          @click="revoke.execute()"
        />
      </template>
      <span
        v-else-if="revokedAt"
        v-tooltip="revokedTooltip"
        class="text-destructive-500 font-bold cursor-pointer w-full text-center hover:underline"
      >
        Yes
      </span>
    </td>
  </tr>
</template>

<script lang="ts" setup>
import clsx from "clsx";
import {
  themeClasses,
  IconButton,
  TruncateWithTooltip,
  Timestamp,
} from "@si/vue-lib/design-system";
import { computed } from "vue";
import { useAsyncState } from "@vueuse/core";
import { apiData } from "@si/vue-lib/pinia";
import { Workspace } from "@/store/workspaces.store";
import { useAuthTokensApi } from "@/store/authTokens.store";
import { AuthTokenWithRealtimeData } from "./AuthTokenList.vue";

const api = useAuthTokensApi();

const props = defineProps<{
  authToken: AuthTokenWithRealtimeData;
  workspace: Readonly<Workspace>;
  active: boolean;
}>();

const emit = defineEmits<{
  (e: "revoked"): void;
  (e: "renamed", newName: string): void;
}>();

/** Action to revoke token */
const revoke = useAsyncState(
  async () => {
    const { workspace, authToken } = props;
    await apiData(api.REVOKE_AUTH_TOKEN(workspace.id, authToken.token.id));
    emit("revoked");
  },
  undefined,
  { immediate: false },
);

const token = computed(() => props.authToken.token);

const createdAt = computed(() => new Date(token.value.createdAt));

const expiresAt = computed(() =>
  token.value.expiresAt ? new Date(token.value.expiresAt) : undefined,
);

const expired = computed(() => props.authToken.isExpired);

const revokedAt = computed(() =>
  token.value.revokedAt ? new Date(token.value.revokedAt) : undefined,
);

const revokedTooltip = computed(() => {
  if (revokedAt.value) {
    return {
      content: revokedAt.value,
      theme: "instant-show",
    };
  }
  return null;
});

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
