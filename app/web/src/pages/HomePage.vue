<template>
  <AppLayout>
    <div class="text-shade-0">
      <div
        v-if="workspacesReqStatus.isPending"
        class="w-full flex flex-col items-center gap-4 p-xl"
      >
        <Icon name="loader" size="2xl" />
        <h2>Loading your workspace(s)...</h2>
      </div>
      <ErrorMessage
        v-else-if="workspacesReqStatus.isError"
        :requestStatus="workspacesReqStatus"
      />
      <div
        v-else-if="workspacesReqStatus.isSuccess"
        class="w-full flex flex-col items-center gap-4 p-xl"
      >
        Finished loading!
      </div>
    </div>
  </AppLayout>
</template>

<script setup lang="ts">
import { watch } from "vue";
import { useRouter, useRoute } from "vue-router";
import { ErrorMessage, Icon } from "@si/vue-lib/design-system";
import { useWorkspacesStore } from "@/store/workspaces.store";
import AppLayout from "@/components/layout/AppLayout.vue";

const router = useRouter();
const route = useRoute();

const workspacesStore = useWorkspacesStore();
const redirectPath = route.query.redirect as string;

const workspacesReqStatus = workspacesStore.getRequestStatus(
  "FETCH_USER_WORKSPACES",
);

async function autoSelectWorkspace() {
  if (workspacesStore.selectedWorkspace) return;

  const workspacePk = workspacesStore.getAutoSelectedWorkspacePk();
  if (!workspacePk) return;

  router.push({
    name: "new-hotness-workspace",
    params: { workspacePk },
  });

  const routeName = "new-hotness-workspace";

  const redirectObject = redirectPath
    ? { path: redirectPath }
    : { name: routeName, params: { workspacePk } };

  await router.replace(redirectObject);
}

watch(
  workspacesReqStatus,
  () => {
    if (workspacesReqStatus.value.isSuccess) autoSelectWorkspace();
  },
  { immediate: true },
);
</script>
