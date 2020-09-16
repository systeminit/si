<template>
  <div class="h-32 application-card">
    <div class="block pt-3 mx-3 text-white" data-cy="application-card-name">
      {{ application.name }}
    </div>

    <div class="flex mt-1">
      <div class="flex flex-row items-start justify-between w-11/12">
        <div class="block w-1/4 h-full pt-1 mx-3 border card-section">
          <ActivityVisualization class="mx-2 mb-2" />
        </div>

        <div class="block w-1/4 h-full pt-1 mx-3 border card-section">
          <ServicesVisualization class="mx-2 mb-2" />
        </div>

        <div class="block w-1/4 h-full pt-1 mx-3 border card-section">
          <SystemsVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
          />
        </div>

        <div class="block w-1/4 h-full pt-1 mx-3 border card-section">
          <ChangeSetVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
          />
        </div>
      </div>

      <div class="relative w-1/12 mt-5 mr-3 text-sm font-bold">
        <button
          class="absolute inset-y-0 right-0 w-6 h-8 text-gray-500 hover:text-white"
          @click="goToApplication(application.id)"
          type="button"
        >
          <chevron-right-icon size="2x" />
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { ChevronRightIcon } from "vue-feather-icons";
import ServicesVisualization from "@/components/visualization/ServicesVisualization.vue";
import SystemsVisualization from "@/components/visualization/SystemsVisualization.vue";
import ChangeSetVisualization from "@/components/visualization/ChangeSetVisualization.vue";
import ActivityVisualization from "@/components/visualization/ActivityVisualization.vue";

export default Vue.extend({
  name: "ApplicationCard",
  props: {
    application: {},
  },
  components: {
    ChevronRightIcon,
    ActivityVisualization,
    ServicesVisualization,
    SystemsVisualization,
    ChangeSetVisualization,
  },
  methods: {
    goToApplication() {
      "/o/:organizationId/w/:workspaceId/a/:applicationId";
      this.workspace = this.$store.getters["workspace/current"];
      this.$router.push({
        name: "applicationDetails",
        params: {
          organizationId: this.workspace.siProperties.organizationId,
          workspaceId: this.workspace.id,
          applicationId: this.application.id,
        },
      });
    },
  },
});
</script>

<style>
.application-card {
  background-color: #292f32;
}

.card-section {
  background-color: #242a2c;
  border-color: #384145;
}
</style>
