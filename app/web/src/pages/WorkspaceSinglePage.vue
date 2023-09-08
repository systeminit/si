<template>
  <AppLayout>
    <Navbar />

    <template v-if="workspacesReqStatus.isPending">
      <div class="flex-grow p-lg flex flex-col items-center gap-4">
        <Icon name="loader" size="2xl" />
        <h2>Loading your workspace(s)...</h2>
      </div>
    </template>
    <template v-else-if="workspacesReqStatus.isError">
      <div class="flex-grow p-lg flex flex-col items-center">
        <ErrorMessage>Error loading your workspaces</ErrorMessage>
      </div>
    </template>

    <!-- by this point we know we have a valid workspace selected and loaded -->
    <template v-else>
      <template
        v-if="
          !changeSetsReqStatus.lastSuccessAt &&
          (!changeSetsReqStatus.isRequested || changeSetsReqStatus.isPending)
        "
      >
        <div class="flex-grow p-lg flex flex-col items-center gap-4">
          <Icon name="loader" size="2xl" />
          <h2>Loading change sets...</h2>
        </div>
      </template>
      <template v-else-if="changeSetsReqStatus.isError">
        <div class="flex-grow p-lg flex flex-col items-center">
          <ErrorMessage>Error loading change sets</ErrorMessage>
        </div>
      </template>

      <!-- all good - either no change set (fix/view) or we have a selected and valid change set -->
      <template v-else>
        <div
          v-if="changeSetsStore.creatingChangeSet"
          class="text-center text-2xl z-100 absolute w-full h-full bg-black bg-opacity-50 flex flex-row justify-center items-center"
        >
          <div class="bg-black text-white w-1/5 rounded-lg">
            <LoadingMessage>Creating Change Set...</LoadingMessage>
          </div>
        </div>
        <div class="w-full h-full flex flex-row relative overflow-hidden">
          <router-view :key="changeSetId" />
        </div>
        <StatusBar :key="changeSetId" class="flex-none" />
      </template>
    </template>
  </AppLayout>
</template>

<script lang="ts" setup>
import { computed, PropType, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import * as _ from "lodash-es";
import { ErrorMessage, Icon, LoadingMessage } from "@si/vue-lib/design-system";
import { useChangeSetsStore } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import { nilId } from "@/utils/nilId";
import AppLayout from "@/components/layout/AppLayout.vue";
import Navbar from "@/components/layout/navbar/Navbar.vue";
import StatusBar from "@/components/StatusBar.vue";

const props = defineProps({
  workspacePk: { type: String, required: true },
  changeSetId: { type: String as PropType<string | "auto"> },
});

const router = useRouter();
const route = useRoute();

const workspacesStore = useWorkspacesStore();
const changeSetsStore = useChangeSetsStore();

const workspacesReqStatus = workspacesStore.getRequestStatus(
  "FETCH_USER_WORKSPACES",
);
const selectedWorkspace = computed(() => workspacesStore.selectedWorkspace);

const changeSetsReqStatus =
  changeSetsStore.getRequestStatus("FETCH_CHANGE_SETS");

// this page is the parent of many child routes so we watch the route rather than use mounted hooks
watch([route, changeSetsReqStatus], handleUrlChange, { immediate: true });

function handleUrlChange() {
  changeSetsStore.creatingChangeSet = false;

  if (!route.name || !changeSetsReqStatus.value.isSuccess) return;

  const changeSetId = route.params.changeSetId as string | undefined;
  if ([undefined, "null", "undefined", "auto"].includes(changeSetId ?? "")) {
    const pk = changeSetsStore.getAutoSelectedChangeSetId();
    router.replace({
      name: route.name, // eslint-disable-line @typescript-eslint/no-non-null-assertion
      params: {
        ...route.params,
        changeSetId: pk === false ? "head" : pk,
      },
    });
    return;
  }

  if ([nilId(), "head"].includes(changeSetId ?? "")) {
    changeSetsStore.selectedChangeSetId = nilId();
  } else if (changeSetId) {
    changeSetsStore.selectedChangeSetId = changeSetId;
  }

  window.localStorage.setItem("tab_group_proposed_right", "actions_proposed");

  if (
    !changeSetId ||
    (changeSetsReqStatus.value.isSuccess &&
      !changeSetsStore.selectedChangeSet &&
      changeSetsStore.selectedChangeSetId)
  ) {
    router.replace({
      name: route.name,
      params: {
        ...route.params,
        changeSetId: "head",
      },
    });
  }
}

watch(
  [selectedWorkspace, workspacesReqStatus],
  () => {
    if (
      workspacesReqStatus.value.isSuccess &&
      selectedWorkspace.value === null
    ) {
      router.replace({
        name: "home",
      });
    }
  },
  {
    immediate: true,
  },
);
</script>
