<template>
  <AppLayout>
    <Navbar />

    <template v-if="workspacesReqStatus.isPending">
      Loading your workspace(s)...
    </template>
    <template v-else-if="workspacesReqStatus.isError">
      <ErrorMessage>Error loading your workspaces</ErrorMessage>
    </template>
    <template v-else-if="!selectedWorkspace">
      <ErrorMessage>Cannot find workspace {{ workspaceId }}</ErrorMessage>
    </template>
    <template v-else>
      <!-- no change set on some routes, otherwise it will only be set if change set is selected and valid -->
      <template v-if="changeSetId === undefined || selectedChangeSet">
        <div class="w-full h-full flex flex-row relative overflow-hidden">
          <router-view :key="selectedChangeSet?.id" />
        </div>
        <StatusBar class="flex-none" />
      </template>
      <template v-else>
        <div class="flex-grow text-white p-lg">
          <template v-if="changeSetsReqStatus.isPending">
            <h2>Loading change sets...</h2>
          </template>
          <template v-else-if="changeSetsReqStatus.isError">
            <ErrorMessage>Error loading change sets</ErrorMessage>
          </template>
          <template v-else>
            <Stack>
              <ErrorMessage
                v-if="
                  changeSetId && !selectedChangeSet && changeSetId !== 'auto'
                "
              >
                Change set {{ changeSetId }} not found
              </ErrorMessage>

              <template v-if="openChangeSets.length">
                <div v-for="changeSet in openChangeSets" :key="changeSet.id">
                  {{ changeSet.name }}
                  <VButton2
                    icon="arrow--right"
                    size="sm"
                    variant="ghost"
                    label="Select"
                    :link-to="{
                      name: 'change-set-home',
                      params: { changeSetId: changeSet.id },
                    }"
                  />
                </div>
                <Divider label="or" />
              </template>

              <template v-if="openChangeSets.length">
                <p>Create a new change set</p>
              </template>
              <template v-else>
                <p>You have no open change sets - please create one.</p>
              </template>

              <div class="flex gap-sm">
                <VormInput
                  v-model="createChangeSetPayload.name"
                  label="Change set name"
                />
                <VormInput type="container">
                  <VButton2 icon="plus-circle" @click="onCreateChangeSet"
                    >Create change set</VButton2
                  >
                </VormInput>
              </div>
            </Stack>
          </template>
        </div>
      </template>
    </template>

    <SiToast />
  </AppLayout>
</template>

<script lang="ts" setup>
import { computed, onBeforeMount, PropType, reactive, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import _ from "lodash";
import Navbar from "@/organisms/Navbar.vue";
import StatusBar from "@/organisms/StatusBar.vue";
import SiToast from "@/atoms/SiToast.vue";
import { ChangeSetId, useChangeSetsStore } from "@/store/change_sets.store";
import VormInput from "@/ui-lib/forms/VormInput.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Divider from "@/ui-lib/layout/Divider.vue";
import { useWorkspacesStore } from "@/store/workspaces.store";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import Stack from "../ui-lib/layout/Stack.vue";
import AppLayout from "./AppLayout.vue";

const props = defineProps({
  workspaceId: { type: Number, required: true },
  changeSetId: { type: [Number, String] as PropType<number | "auto"> },
});

const router = useRouter();
const route = useRoute();

const workspacesStore = useWorkspacesStore();
const changeSetsStore = useChangeSetsStore();

const workspacesReqStatus = workspacesStore.getRequestStatus(
  "FETCH_USER_WORKSPACES",
);
const selectedWorkspace = computed(() => workspacesStore.selectedWorkspace);

const openChangeSets = computed(() => changeSetsStore.openChangeSets);
const changeSetsReqStatus =
  changeSetsStore.getRequestStatus("FETCH_CHANGE_SETS");
const selectedChangeSet = computed(() => changeSetsStore.selectedChangeSet);

// this page is the parent of many child routes so we watch the route rather than use mounted hooks
watch([route, changeSetsReqStatus], handleAutoRedirect);
onBeforeMount(handleAutoRedirect);

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

function handleAutoRedirect() {
  if (props.changeSetId !== "auto" || !changeSetsReqStatus.value.isSuccess)
    return;

  const autoSelectChangeSetId = changeSetsStore.getAutoSelectedChangeSetId();
  if (autoSelectChangeSetId) {
    routeToChangeSet(autoSelectChangeSetId, true);
  }
}

const createChangeSetPayload = reactive({
  name: "",
});
async function onCreateChangeSet() {
  const req = await changeSetsStore.CREATE_CHANGE_SET(
    createChangeSetPayload.name,
  );
  if (req.result.success) routeToChangeSet(req.result.data.changeSet.id);
}
</script>
