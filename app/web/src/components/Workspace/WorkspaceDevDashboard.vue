<template>
  <div
    class="w-full h-full flex flex-col items-center relative overflow-hidden dark:bg-neutral-800 dark:text-shade-0 bg-neutral-50 text-neutral-900"
  >
    <Stack spacing="lg">
      <span class="flex flex-row mt-10 font-bold text-3xl">Dev Dashboard</span>
      <Stack>
        <h2 class="font-bold text-lg">FRONTEND</h2>
        <Inline spacing="md">
          <div>
            <span class="font-bold">Branch: </span>
            <a :href="getGithubBranchLink(webBranch)" class="text-action-400 hover:underline" target="_blank">
              {{ webBranch }}
            </a>
          </div>
          <div>
            <span class="font-bold">Sha: </span>
            <a :href="getGithubShaLink(webSha)" class="text-action-400 hover:underline" target="_blank">
              {{ webSha }}
            </a>
          </div>
        </Inline>
        <div class="text-md font-bold text-action-400 hover:underline">
          <RouterLink :to="{ name: 'svg' }">
            <div class="text-2xl pb-2xs">Debug Design Reference Page</div>
            <div class="italic text-sm">(see all Icons, EmptyStateIcons, other SVGs, and Semantic Sizes/Colors)</div>
          </RouterLink>
        </div>
      </Stack>

      <Stack>
        <h2>API</h2>
        <ErrorMessage :requestStatus="apiVersionReqStatus" />
        <template v-if="apiVersionReqStatus.isPending">Loading...</template>
        <template v-else-if="apiVersionReqStatus.isSuccess && apiSha && apiBranch">
          <Inline spacing="md">
            <div>
              Branch:
              <a :href="getGithubBranchLink(apiBranch)" class="text-action-400" target="_blank">{{ apiBranch }}</a>
            </div>
            <div>
              Sha:
              <a :href="getGithubShaLink(apiSha)" class="text-action-400" target="_blank">{{ apiSha }}</a>
            </div>
          </Inline>
        </template>
      </Stack>
    </Stack>
  </div>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount } from "vue";
import { ErrorMessage, Inline, Stack } from "@si/vue-lib/design-system";
import { useSystemStatusStore } from "@/store/system_status.store";

const systemStatusStore = useSystemStatusStore();

onBeforeMount(() => {
  systemStatusStore.CHECK_CURRENT_API_VERSION();
});

const apiVersionReqStatus = systemStatusStore.getRequestStatus("CHECK_CURRENT_API_VERSION");

const apiBranch = computed(() => systemStatusStore.apiGitBranch);
const apiSha = computed(() => systemStatusStore.apiGitSha);

const webBranch = computed(() => systemStatusStore.webGitBranch);
const webSha = computed(() => systemStatusStore.webGitSha);

function getGithubBranchLink(branch: string) {
  return `https://github.com/systeminit/si/tree/${branch}`;
}

function getGithubShaLink(sha: string) {
  return `https://github.com/systeminit/si/commit/${sha}`;
}
</script>
