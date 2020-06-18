<template>
  <div class="h-24 application-card">
    
    <div class="block mx-3 pt-3 text-white">
    {{ application.name }}
    </div>

    <div class="flex">
      <div class="flex flex-row w-11/12 items-start pt-1 justify-between">

        <div class="block mx-3 pt-1 w-1/4 h-full card-section">
          <LastUpdatedVisualization/>
        </div>

        <div class="block mx-3 pt-1 w-1/4 h-full card-section">
          <ServicesVisualization/>
        </div>

        <div class="block mx-3 pt-1 w-1/4 h-full card-section">
          <SystemsVisualization/>
        </div>

        <div class="block mx-3 pt-1 w-1/4 h-full card-section">
          <ChangeSetVisualization/>
        </div>

      </div>

      <div class="relative mx-3 pt-1 w-1/12 text-sm font-bold">

        <button
          class="absolute inset-y-0 right-0 bg-teal-700 w-6 h-8 text-white hover:bg-teal-600"
          @click="goToApplication(application.id)"
          type="button"
        >
          <chevron-right-icon size="1.5x" class="text-right"></chevron-right-icon>
        </button>
    </div>

    </div>
  </div>
</template>

<script>
import { ChevronRightIcon } from 'vue-feather-icons'
import ServicesVisualization from "@/components/visualization/ServicesVisualization.vue"
import SystemsVisualization from "@/components/visualization/SystemsVisualization.vue"
import ChangeSetVisualization from "@/components/visualization/ChangeSetVisualization.vue"
import LastUpdatedVisualization from "@/components/visualization/LastUpdatedVisualization.vue"

export default {
  name: "ApplicationCard",
  props: {
    application: {},
    session: {}
  },
  components: {
    ChevronRightIcon,
    LastUpdatedVisualization,
    ServicesVisualization,
    SystemsVisualization,
    ChangeSetVisualization
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
  }
}
</script>

<style>
  
.application-card {
  background-color: #181B19
}

.card-section {
  background-color: #222526

}


</style>