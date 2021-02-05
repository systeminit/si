<template>
  <div class="h-32 bg-gray-800">
    <div class="block pt-3 mx-3 text-white">
      {{ application.name }}
    </div>

    <div class="flex mt-1">
      <div class="flex flex-row items-start justify-between w-11/12">
        <div
          class="block w-1/4 h-full pt-1 mx-3 border bg-gray-900 border-gray-400"
        >
          Activity Visualization
          <!-- <ActivityVisualization class="mx-2 mb-2" /> -->
        </div>

        <div
          class="block w-1/4 h-full pt-1 mx-3 border bg-gray-900 border-gray-400"
        >
          Services Visualization
          <!-- 
          <ServicesVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
            inEditor="false"
          /> 
          -->
        </div>

        <div
          class="block w-1/4 h-full pt-1 mx-3 border bg-gray-900 border-gray-400"
        >
          Systems Visualization
          <!--
          <SystemsVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
          />
          -->
        </div>

        <div
          class="block w-1/4 h-full pt-1 mx-3 border bg-gray-900 border-gray-400"
        >
          <ChangeSetCounts
            :changeSetCounts="applicationEntry.changeSetCounts"
            class="mx-2 mb-2"
          />
          <!--
          <ChangeSetVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
          />
          -->
        </div>
      </div>

      <div class="relative w-1/12 mt-5 mr-3 text-sm font-bold">
        <button
          class="absolute inset-y-0 right-0 w-6 h-8 text-gray-500 hover:text-white"
          @click="goToApplication()"
          type="button"
        >
          <chevron-right-icon size="2x" />
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { IApplicationListEntry } from "@/store/modules/application";
import { RawLocation } from "vue-router";

import { ChevronRightIcon } from "vue-feather-icons";
import ChangeSetCounts from "@/molecules/ChangeSetCounts.vue";

export default Vue.extend({
  name: "ApplicationDetailCard",
  props: {
    applicationEntry: {
      type: Object as PropType<IApplicationListEntry>,
    },
    linkTo: {
      type: Object as PropType<RawLocation>,
    },
  },
  components: {
    ChevronRightIcon,
    ChangeSetCounts,
  },
  computed: {
    application(): IApplicationListEntry["application"] {
      return this.applicationEntry.application;
    },
  },
  methods: {
    goToAppliction() {
      this.$router.push(this.linkTo);
    },
  },
});
</script>
