<template>
  <div
    v-if="user && user.emailVerified"
    class="overflow-hidden flex flex-col gap-sm"
  >
    <!-- TITLE -->
    <div class="flex flex-col">
      <div class="text-lg font-bold">{{ workspaceTitle }}</div>
      <div class="text-xs">
        From here you can log into any of your workspaces.
      </div>
      <!-- <VButton
        v-if="featureFlagsStore.CREATE_WORKSPACES"
        :linkTo="{ name: 'workspace-settings', params: { workspaceId: 'new' } }"
        icon="plus"
        label="Create Workspace"
      /> -->
    </div>

    <!-- SEARCH -->
    <!-- TODO(Wendy) - add variant for SiSearch for the auth portal to match Mark's Figma design -->
    <SiSearch placeholder="Search workspaces..." disableFilters />

    <!-- HELP BANNER-->
    <div
      :class="
        clsx(
          'p-xs border rounded',
          themeClasses(
            'bg-shade-0 border-neutral-200',
            'bg-neutral-800 border-neutral-700',
          ),
        )
      "
    >
      If you have questions or need help, join us on
      <a
        class="text-action-500 dark:text-action-400 font-bold hover:underline"
        href="https://discord.gg/system-init"
        target="_blank"
      >
        Discord
      </a>
      or visit our
      <a
        class="text-action-500 dark:text-action-400 font-bold hover:underline"
        href="https://docs.systeminit.com"
        target="_blank"
        >docs site</a
      >.
    </div>

    <!-- TIER AND BILLING INFO -->
    <div class="flex flex-row gap-md">
      <!-- Resource Count -->
      <InfoCard>
        <template #title>
          <Icon
            name="check-hex"
            class="flex-none text-success-600 dark:text-success-500"
            size="lg"
          />
          <div class="text-lg font-bold flex-grow">Resources</div>
        </template>
        <template #infoRow1>90</template>
        <template #infoRow2>under management</template>
      </InfoCard>
      <!-- Pricing Tier -->
      <InfoCard title="Tier">
        <template #infoRow1>
          <div
            :class="
              clsx(
                'rounded text-sm px-xs py-2xs my-2xs w-fit',
                themeClasses(
                  'bg-success-600 text-shade-0',
                  'bg-success-500 text-shade-100',
                ),
              )
            "
          >
            FREE TRIAL
          </div>
        </template>
        <template #infoRow2>16 days remaining</template>
      </InfoCard>
      <!-- Invoice Info -->
      <InfoCard title="Total Invoiced">
        <template #infoRow1>$6.66</template>
        <template #infoRow2>
          <div class="flex flex-row gap-xs">
            <div>this month</div>
            <div>|</div>
            <RouterLink
              :to="{ name: 'billing' }"
              class="underline text-action-400"
            >
              view billing
            </RouterLink>
            <div>|</div>
            <div class="underline text-action-400">
              <!-- TODO - this needs to be a working link! -->
              payment method
            </div>
          </div>
        </template>
      </InfoCard>
    </div>

    <!-- WORKSPACES LIST -->

    <RequestStatusMessage
      v-if="
        loadWorkspacesReqStatus.isPending || loadWorkspacesReqStatus.isError
      "
      message="Loading Workspaces..."
      :requestStatus="loadWorkspacesReqStatus"
    />

    <div v-else-if="loadWorkspacesReqStatus.isSuccess" class="flex flex-col">
      <div class="flex flex-row items-center">
        <!-- TODO(Wendy) - this is where the filtering and sorting bar goes -->
      </div>

      <div class="grid grid-cols-4 gap-sm">
        <div
          :class="
            clsx(
              'group/newworkspace',
              'flex flex-col items-center justify-center gap-sm rounded border-2 border-dashed hover:border-solid cursor-pointer',
              themeClasses(
                'border-action-200 hover:border-action-500 hover:bg-action-200 active:bg-action-400 active:border-shade-100',
                'border-action-900 hover:border-action-400 hover:bg-action-900 active:bg-action-400 active:border-shade-0',
              ),
            )
          "
          @click="createNewWorkspace"
        >
          <div
            :class="
              clsx(
                'rounded-lg p-xs text-shade-0 bg-action-400 group-hover/newworkspace:bg-action-600',
              )
            "
          >
            <Icon name="git-branch-plus" rotate="left" size="lg" />
          </div>
          <div
            :class="
              clsx(
                'select-none',
                themeClasses('group-active/newworkspace:text-shade-0', ''),
              )
            "
          >
            Create New Workspace
          </div>
        </div>
        <WorkspaceLinkWidget
          v-for="workspace in sortedWorkspaces(workspaces)"
          :key="workspace.id"
          :workspaceId="workspace.id"
        />
      </div>
    </div>
  </div>
  <div v-else>
    You will not be able to use System Initiative until you verify your email.
  </div>
</template>

<script lang="ts" setup>
import { computed, watch } from "vue";
import {
  Icon,
  SiSearch,
  RequestStatusMessage,
  themeClasses,
} from "@si/vue-lib/design-system";
import { useHead } from "@vueuse/head";
import { RouterLink, useRouter } from "vue-router";
import clsx from "clsx";
import { useAuthStore } from "@/store/auth.store";
import { useWorkspacesStore, Workspace } from "@/store/workspaces.store";
import WorkspaceLinkWidget from "@/components/WorkspaceLinkWidget.vue";
import InfoCard from "../components/InfoCard.vue";

const authStore = useAuthStore();
const workspacesStore = useWorkspacesStore();
const router = useRouter();

const workspaces = computed(() => workspacesStore.workspaces);
// function sortedWorkspaces(workspaces: Workspace[]): Workspace[] {
//   return workspaces.sort((a, b) => {
//     // First, prioritize "SI" instanceEnvType
//     if (a.instanceEnvType === "SI" && b.instanceEnvType !== "SI") {
//       return -1;
//     }
//     if (a.instanceEnvType !== "SI" && b.instanceEnvType === "SI") {
//       return 1;
//     }

//     // If both are "SI" or both are not "SI", sort by displayName
//     return a.displayName.localeCompare(b.displayName);
//   });
// }
function sortedWorkspaces(workspaces: Workspace[]): Workspace[] {
  return workspaces.sort((a, b) => {
    // 1. Sort by isDefault (true comes first)
    if (a.isDefault !== b.isDefault) {
      return a.isDefault ? -1 : 1;
    }

    // 2. Sort by isFavourite (true comes first)
    if (a.isFavourite !== b.isFavourite) {
      return a.isFavourite ? -1 : 1;
    }

    // 3. Sort by instanceEnvType (SI comes first, then REMOTE, then LOCAL)
    if (a.instanceEnvType !== b.instanceEnvType) {
      const envTypeOrder = { SI: 0, PRIVATE: 1, LOCAL: 2 };
      return envTypeOrder[a.instanceEnvType] - envTypeOrder[b.instanceEnvType];
    }

    // 4. If all above are equal, sort by displayName
    return a.displayName.localeCompare(b.displayName);
  });
}
const workspaceCount = computed(() => workspaces.value.length);
const workspaceTitle = computed(() => {
  if (workspaceCount.value > 1) return `${workspaceCount.value} Workspaces`;
  else if (workspaceCount.value === 1) return "One Workspace";
  else return "Create A Workspace";
});

const createNewWorkspace = async () => {
  await router.push({
    name: "workspace-settings",
    params: { workspaceId: "new" },
  });
};

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
