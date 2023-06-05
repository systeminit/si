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
        <ErrorMessage>Cannot find workspace {{ workspacePk }}</ErrorMessage>
      </div>
    </template>

    <!-- by this point we know we have a valid workspace selected and loaded -->
    <template v-else>
      <template v-if="changeSetsReqStatus.isPending">
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

      <!-- tried to autoselect but failed -->
      <template
        v-else-if="changeSetId === 'auto' && selectedChangeSet === null"
      >
        <div class="w-full h-full flex flex-row relative overflow-hidden">
          <PlaceholderComposeView />
        </div>
        <!-- <ErrorMessage>Auto select failed</ErrorMessage>
        <p>TODO: show create/select modal</p> -->
      </template>

      <!-- <template
        v-else-if="
          selectedChangeSet &&
          selectedChangeSet?.status !== ChangeSetStatus.Open
        "
      >
        <ErrorMessage>Sorry, this change set is no longer open</ErrorMessage>
      </template> -->

      <!-- change set id in the URL, but it is invalid -->
      <template v-else-if="changeSetId && !selectedChangeSet">
        <ErrorMessage>Change set "{{ changeSetId }}" not found</ErrorMessage>
      </template>

      <!-- all good - either no change set (fix/view) or we have a selected and valid change set -->
      <template v-else>
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
import { ErrorMessage, Icon } from "@si/vue-lib/design-system";
import {
  ChangeSetId,
  useChangeSetsStore,
  changeSetIdNil,
} from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import AppLayout from "@/components/layout/AppLayout.vue";
import Navbar from "@/components/layout/navbar/Navbar.vue";
import PlaceholderComposeView from "@/components/layout/PlaceholderComposeView.vue";
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
  const changeSetId = route.params.changeSetId as string | undefined;
  // if "auto", we do our best to autoselect, and show a selection screen otherwise
  if (changeSetId === "auto") {
    tryAutoSelect();
    // if undefined, that means the route has no changeSetId param, so we select "head"
  } else if (changeSetId === undefined) {
    changeSetsStore.selectedChangeSetId = changeSetIdNil();
  } else {
    changeSetsStore.selectedChangeSetId = changeSetId;
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
    // only clear the selected change set id if we are on "auto" mode and we can't automatically select
    changeSetsStore.selectedChangeSetId = null;
  }
}
</script>
