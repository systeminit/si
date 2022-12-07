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
    <template v-else-if="!selectedWorkspace">
      <div class="flex-grow p-lg flex flex-col items-center">
        <ErrorMessage>Cannot find workspace {{ workspaceId }}</ErrorMessage>
      </div>
    </template>
    <template v-else>
      <!-- no change set on some routes, otherwise it will only be set if change set is selected and valid -->
      <template v-if="changeSetId === undefined || selectedChangeSet">
        <div class="w-full h-full flex flex-row relative overflow-hidden">
          <router-view :key="selectedChangeSet?.id" />
        </div>
        <StatusBar :key="selectedChangeSet?.id" class="flex-none" />
      </template>
      <template v-else-if="changeSetsReqStatus.isPending">
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
      <template v-else>
        <div class="w-full h-full flex flex-row relative overflow-hidden">
          <router-view :key="route.fullPath" />
          <!-- without a key the router view doesn't update on route change, which we want here -->
        </div>
        <StatusBar :key="0" class="flex-none" />
      </template>
    </template>
  </AppLayout>
</template>

<script lang="ts" setup>
import { computed, PropType, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import _ from "lodash";
import StatusBar from "@/organisms/StatusBar.vue";
import {
  ChangeSetId,
  useChangeSetsStore,
  changeSetIdNil,
} from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import AppLayout from "@/layout/AppLayout.vue";
import Navbar from "@/layout/navbar/Navbar.vue";
import Icon from "@/ui-lib/icons/Icon.vue";

const props = defineProps({
  workspaceId: { type: String, required: true },
  changeSetId: { type: [String, String] as PropType<string | "auto"> },
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
const selectedChangeSet = computed(() => changeSetsStore.selectedChangeSet);

// this page is the parent of many child routes so we watch the route rather than use mounted hooks
watch(changeSetsReqStatus, handleChangeSetsLoaded);
watch(route, handleUrlChange, { immediate: true });

function routeToChangeSet(id: ChangeSetId, replace = false) {
  // reroutes to a specific changeset but keeps the same route name
  // so we can go from /auto/some-specific-page -> 1/some-specific-page
  router[replace ? "replace" : "push"]({
    name: route.name!, // eslint-disable-line @typescript-eslint/no-non-null-assertion
    params: {
      ...route.params,
      changeSetId: id,
    },
  });
}

function handleUrlChange() {
  // if id looks like a string, we set it in the store
  if (props.changeSetId === "auto") {
    tryAutoSelect();
  } else if (_.isString(props.changeSetId)) {
    changeSetsStore.selectedChangeSetId = props.changeSetId;
    // if undefined, that means the route has no changeSetId param, so we select "head"
  } else if (props.changeSetId === undefined) {
    changeSetsStore.selectedChangeSetId = changeSetIdNil();
    // if "auto", we do our best to autoselect, and show a selection screen otherwise
  }
}
function handleChangeSetsLoaded() {
  if (changeSetsReqStatus.value.isSuccess && props.changeSetId === "auto") {
    tryAutoSelect();
  }
}

// gets called on url change when id is "auto", and also when change set's are loaded
function tryAutoSelect() {
  const autoSelectChangeSetId = changeSetsStore.getAutoSelectedChangeSetId();
  if (autoSelectChangeSetId) {
    routeToChangeSet(autoSelectChangeSetId, true);
  } else {
    // only clear the selected change set id if we are on "auto" mode and we can't autoamtically select
    changeSetsStore.selectedChangeSetId = null;
  }
}
</script>
