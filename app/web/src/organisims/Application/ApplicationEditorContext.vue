<template>
  <div class="flex flex-col w-full pb-2 header-background">
    <div class="flex justify-between mt-2">
      <div class="flex items-center">
        <button
          class="focus:outline-none"
          data-cy="application-details-toggle"
          @click="toggleDetails"
        >
          <VueFeather
            v-if="showDetails"
            type="chevron-down"
            size="1.5em"
            class="text-gray-300"
          />
          <VueFeather
            v-else
            type="chevron-right"
            size="1.5rem"
            class="text-gray-300"
          />
        </button>
      </div>

      <!--
      <MenuSummary
        v-if="!showDetails && application"
        :applicationId="application.id"
      />
      -->

      <div v-if="!showDetails" class="flex mr-2">
        <!--
        <EditorMenuBar
          :workspace="currentWorkspace"
          :application="application"
        />
        -->
      </div>
    </div>
    <div
      v-if="showDetails"
      class="flex w-full h-full pb-2 details-panel-background"
      data-cy="application-details-extended"
    >
      <div class="w-1/5 h-full py-2 mx-2">
        <ApplicationActivitySummary :application-id="application.id" />
      </div>

      <div class="w-1/5 h-full py-2 mx-2">
        <ApplicationServicesSummary :application-id="application.id" />
      </div>

      <div class="w-1/5 h-full py-2 mx-2">
        <ApplicationComputingResourcesSummary
          :application-id="application.id"
        />
      </div>

      <div class="w-1/5 h-full py-2 mx-2">
        <ApplicationProviderSummary :application-id="application.id" />
      </div>

      <div class="w-1/5 h-full py-2 mx-2">
        <ApplicationChangesSummary :application-id="application.id" />
      </div>
    </div>

    <div v-if="showDetails" class="flex justify-end mt-1 mr-2">
      <div class="flex items-center justify-end">
        <!--
        <EditorMenuBar
          :workspace="currentWorkspace"
          :application="application"
        />
        -->
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";

import ApplicationActivitySummary from "@/organisims/Application/Summary/ApplicationActivitySummary.vue";
import ApplicationServicesSummary from "@/organisims/Application/Summary/ApplicationServicesSummary.vue";
import ApplicationComputingResourcesSummary from "@/organisims/Application/ApplicationComputingResourcesSummary.vue";
import ApplicationProviderSummary from "@/organisims/Application/Summary/ApplicationProviderSummary.vue";
import ApplicationChangesSummary from "@/organisims/Application/Summary/ApplicationChangesSummary.vue";
import VueFeather from "vue-feather";

const showDetails = ref<boolean>(false);
const toggleDetails = () => {
  showDetails.value = !showDetails.value;
};

// TODO: Need to pass a real application in to this component, and down through to other places.
const application = {
  id: 1,
};
</script>

<style scoped>
.details-panel {
  border: 1px solid #464753;
  background-color: #101010;
}

.details-panel-title {
  /* @apply font-normal text-xs; */
  font-weight: 400;
  font-size: 0.75rem;
  line-height: 1rem;
}

.details-panel-background {
  background-color: #171717;
}

.header-background {
  background-color: #171717;
}

.title-background {
  background-color: #292929;
}
</style>
