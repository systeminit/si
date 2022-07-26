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
  </div>
</template>

<script setup lang="ts">
import { watch } from "vue";
import Editor from "@/organisms/Editor.vue";
import { refFrom, untilUnmounted } from "vuse-rx";
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
</script>
