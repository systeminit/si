<template>
  <div class="h-32 application-card">
    <div class="block mx-3 pt-3 text-white">
      {{ application.name }}
    </div>

    <div class="flex mt-1">
      <div class="flex flex-row w-11/12 items-start justify-between">
        <div class="block mx-3 pt-1 w-1/4 h-full card-section border">
          <ActivityVisualization class="mx-2 mb-2" />
        </div>

        <div class="block mx-3 pt-1 w-1/4 h-full card-section border">
          <ServicesVisualization class="mx-2 mb-2" />
        </div>

        <div class="block mx-3 pt-1 w-1/4 h-full card-section border">
          <SystemsVisualization class="mx-2 mb-2" />
        </div>

        <div class="block mx-3 pt-1 w-1/4 h-full card-section border">
          <ChangeSetVisualization class="mx-2 mb-2" />
        </div>
      </div>

      <div class="relative mr-3 mt-5 w-1/12 text-sm font-bold">
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

<script>
import { ChevronRightIcon } from "vue-feather-icons";
import ServicesVisualization from "@/components/visualization/ServicesVisualization.vue";
import SystemsVisualization from "@/components/visualization/SystemsVisualization.vue";
import ChangeSetVisualization from "@/components/visualization/ChangeSetVisualization.vue";
import ActivityVisualization from "@/components/visualization/ActivityVisualization.vue";

export default {
  name: "ApplicationCard",
  props: {
    application: {},
    session: {},
  },
  components: {
    ChevronRightIcon,
    ActivityVisualization,
    ServicesVisualization,
    SystemsVisualization,
    ChangeSetVisualization,
  },
  methods: {
    goToApplication(appId) {
      "/o/:organizationId/w/:workspaceId/a/:applicationId";
      this.$router.push({
        name: "applicationDetails",
        params: {
          organizationId: this.session.organizationId,
          workspaceId: this.session.workspaceId,
          applicationId: appId,
        },
      });
    },
  },
};
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
