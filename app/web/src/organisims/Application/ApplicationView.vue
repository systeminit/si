<template>
  <div
    v-if="application"
    id="application-editor"
    class="flex flex-col w-full h-full select-none"
  >
    <div class="flex flex-col w-full h-full">
      <StatusBar />
      <div class="flex flex-col w-full h-16">
        <div class="flex justify-end h-full">
          <div class="flex mr-2 items-center">
            <!-- <SiSystemHeader /> -->
            <SiChangeSetHeader />
          </div>
        </div>
      </div>
      <!-- TODO: Reimplement this context
      <ApplicationEditorContext />
      -->
      <div id="editor" class="flex w-full h-full overflow-hidden">
        <Editor />
      </div>
      <!--
      <EventBar />
      -->
      <!--
    <div id="eventBar" class="w-full">
      <EventBar />
    </div>
      -->
    </div>
    <!-- this one is extra -->

    <SiModal v-model="unsavedChangesModalShow" name="unsavedChanges">
      <template #title> Unsaved Changes</template>
      <template #body>
        <div class="flex flex-col items-center w-full h-full mb-2">
          <div class="text-base font-normal text-red-500">
            You have unsaved changes!
          </div>
          <div class="text-sm text-white">Are you sure you want to leave?</div>
        </div>
      </template>
      <template #buttons>
        <SiButton
          size="sm"
          label="leave"
          class="mx-1"
          :icon="null"
          kind="cancel"
          @click="leave"
        />
        <SiButton
          size="sm"
          label="stay"
          class="mx-1"
          :icon="null"
          kind="save"
          @click="stay"
        />
      </template>
    </SiModal>
  </div>
</template>

<script setup lang="ts">
import { onUnmounted, ref, watch } from "vue";

import Editor from "@/organisims/Editor.vue";
import SiModal from "@/molecules/SiModal.vue";
import SiButton from "@/atoms/SiButton.vue";
import {
  onBeforeRouteLeave,
  RouteLocationNormalized,
  useRouter,
} from "vue-router";
import { ChangeSetService } from "@/service/change_set";
import { refFrom, untilUnmounted } from "vuse-rx";
import { switchToHead } from "@/service/change_set/switch_to_head";
import _ from "lodash";
import { ApplicationService } from "@/service/application";
import { setIfError } from "@/service/global_error";
import { Component } from "@/api/sdf/dal/component";
import StatusBar from "@/molecules/StatusBar.vue";
import SiChangeSetHeader from "@/molecules/SiChangeSetHeader.vue";
//import SiSystemHeader from "@/molecules/SiSystemHeader.vue";

const props = defineProps({
  applicationId: {
    type: Number,
    required: true,
  },
});

watch(
  () => props.applicationId,
  (newApplicationId, _oldValue) => {
    setIfError(
      untilUnmounted(
        ApplicationService.setCurrentApplication({
          applicationId: newApplicationId,
        }),
      ),
    );
  },
  { immediate: true },
);

const application = refFrom<Component | null>(
  ApplicationService.currentApplication(),
);

const editMode = refFrom(ChangeSetService.currentEditMode());
const navDestination = ref<RouteLocationNormalized | null>(null);
const unsavedChangesModalShow = ref<boolean>(false);
const reallyLeave = ref<boolean>(false);

const router = useRouter();
const leave = () => {
  if (navDestination.value) {
    unsavedChangesModalShow.value = false;
    reallyLeave.value = true;
    switchToHead();
    router
      .push(navDestination.value)
      .catch((err) => console.log("route error", { err }));
  }
};

const stay = () => {
  unsavedChangesModalShow.value = false;
  navDestination.value = null;
};

onBeforeRouteLeave((to, _from) => {
  if (editMode.value && reallyLeave.value === false) {
    if (!_.isNull(to)) {
      navDestination.value = to;
      unsavedChangesModalShow.value = true;
      return false;
    }
  }
  return true;
});

onUnmounted(() => {
  ApplicationService.clearCurrentApplication();
  ChangeSetService.switchToHead();
});
</script>
