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

    <template
      v-else-if="workspacesStore.urlSelectedWorkspaceId && !selectedWorkspace"
    >
      <div class="flex-grow p-lg flex flex-col items-center">
        <ErrorMessage
          >Bad workspace id - please select a workspace from the
          dropdown</ErrorMessage
        >
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
      <template
        v-else-if="changeSetsStore.urlSelectedChangeSetId && !selectedChangeSet"
      >
        <div class="flex-grow p-lg flex flex-col items-center">
          <ErrorMessage
            >Change set not found -
            {{ changeSetsStore.urlSelectedChangeSetId }}</ErrorMessage
          >
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
          <router-view :key="changeSetsStore.selectedChangeSet?.id" />
        </div>
        <StatusBar
          :key="changeSetsStore.urlSelectedChangeSetId"
          class="flex-none"
        />
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
import AppLayout from "@/components/layout/AppLayout.vue";
import Navbar from "@/components/layout/navbar/Navbar.vue";
import StatusBar from "@/components/StatusBar/StatusBar.vue";

const props = defineProps({
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
const selectedChangeSet = computed(() => changeSetsStore.selectedChangeSet);

const changeSetsReqStatus =
  changeSetsStore.getRequestStatus("FETCH_CHANGE_SETS");

// this page is the parent of many child routes so we watch the route rather than use mounted hooks
watch([route, changeSetsReqStatus], handleUrlChange, { immediate: true });

// TODO: this logic needs some work
function handleUrlChange() {
  changeSetsStore.creatingChangeSet = false;

  if (!route.name || !changeSetsReqStatus.value.isSuccess) return;

  const changeSetId = route.params.changeSetId as string | undefined;
  if ([undefined, "null", "undefined", "auto"].includes(changeSetId ?? "")) {
    const id = changeSetsStore.getAutoSelectedChangeSetId();

    router.replace({
      name: route.name, // eslint-disable-line @typescript-eslint/no-non-null-assertion
      params: {
        ...route.params,
        changeSetId:
          id === false || id === changeSetsStore.headChangeSetId ? "head" : id,
      },
      query: { ...route.query },
    });
    return;
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
      query: route.query,
    });
  }
}
</script>
