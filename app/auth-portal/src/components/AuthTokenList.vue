<template>
  <Stack spacing="xs">
    <div class="text-lg font-bold line-clamp-3 break-words">{{ title }}</div>
    <table class="w-full table-fixed">
      <thead>
        <tr
          :class="
            clsx(
              'children:p-xs children:font-bold text-left text-xs uppercase',
              themeClasses(
                'bg-neutral-300 text-shade-100',
                'bg-shade-100 text-shade-0',
              ),
            )
          "
        >
          <th scope="col">Name</th>
          <th scope="col" class="w-52">Created</th>
          <th scope="col" class="w-52">
            {{ active ? "Expires" : "Expiration" }}
          </th>
          <th
            v-if="workspace.role === 'OWNER'"
            class="w-24 text-center"
            scope="col"
          >
            <template v-if="active">Revoke</template>
            <template v-else>Revoked</template>
          </th>
        </tr>
      </thead>
      <tbody>
        <AuthTokenListItem
          v-for="authToken of authTokens"
          :key="authToken.token.id"
          :authToken="authToken"
          :workspace="workspace"
          :active="active"
          @revoked="emit('revoked', authToken.token.id)"
          @renamed="(newName) => emit('renamed', authToken.token.id, newName)"
        />
      </tbody>
    </table>
  </Stack>
</template>

<script lang="ts" setup>
import { computed, PropType } from "vue";
import { Stack, themeClasses } from "@si/vue-lib/design-system";
import clsx from "clsx";
import { Workspace } from "@/store/workspaces.store";
import AuthTokenListItem from "@/components/AuthTokenListItem.vue";
import { AuthToken } from "@/store/authTokens.store";

export interface AuthTokenWithRealtimeData {
  token: AuthToken;
  isExpired: boolean;
  isActive: boolean;
}

const props = defineProps({
  workspace: { type: Object as PropType<Workspace>, required: true },
  authTokens: { type: Array<AuthTokenWithRealtimeData>, default: [] },
  active: { type: Boolean },
});

const title = computed(() =>
  props.active ? "Active Tokens" : "Inactive Tokens",
);

const emit = defineEmits<{
  (e: "revoked", id: string): void;
  (e: "renamed", id: string, newName: string): void;
}>();
</script>
