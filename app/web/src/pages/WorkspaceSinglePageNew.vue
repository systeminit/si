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
import { useChangeSetsStore, changeSetIdNil } from "@/store/change_sets.store";
import { useWorkspacesStore } from "@/store/workspaces.store";
import AppLayout from "@/components/layout/AppLayout.vue";
import Navbar from "@/components/layout/navbar/Navbar2.vue";
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
watch(route, handleUrlChange, { immediate: true });

function handleUrlChange() {
  // if "auto", we do our best to autoselect, and show a selection screen otherwise
  if (props.changeSetId === "auto") {
    changeSetsStore.selectedChangeSetId = null;
    // if undefined, that means the route has no changeSetId param, so we select "head"
  } else if (props.changeSetId === undefined) {
    changeSetsStore.selectedChangeSetId = changeSetIdNil();
  } else {
    changeSetsStore.selectedChangeSetId = props.changeSetId;
  }

  if (
    !changeSetsStore.selectedChangeSet &&
    changeSetsStore.selectedChangeSetId
  ) {
    router.replace({
      name: route.name!, // eslint-disable-line @typescript-eslint/no-non-null-assertion
      params: {
        ...route.params,
        changeSetId: "auto",
      },
    });
  }
}
</script>
