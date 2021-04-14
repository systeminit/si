<template>
  <div class="flex flex-col justify-center w-full application-card">
    <div class="py-1 pl-2 card-header">
      {{ application.name }}
    </div>

    <div class="flex w-full my-3">
      <div class="flex flex-col w-full ml-2 visualization-card">
        <div class="mx-2 mt-2 visualization-title">
          Activity
          <!-- <ActivityVisualization class="mx-2 mb-2" /> -->
        </div>
      </div>

      <div class="flex flex-col w-full ml-2 visualization-card">
        <div class="mx-2 mt-2 visualization-title">
          Services
          <!-- 
            <ServicesVisualization
              class="mx-2 mb-2"
              :applicationId="application.id"
              inEditor="false"
            /> 
            -->
        </div>
      </div>

      <div class="flex flex-col w-full ml-2 visualization-card">
        <div class="mx-2 mt-2 visualization-title">
          Systems
          <!--
            <SystemsVisualization
              class="mx-2 mb-2"
              :applicationId="application.id"
            />
            -->
        </div>
      </div>

      <div class="flex flex-col w-full ml-2 visualization-card">
        <div class="mx-2 mt-2 visualization-title">
          Changes
        </div>
        <div class="mx-4 my-1">
          <ChangeSetCounts
            :changeSetCounts="applicationEntry.changeSetCounts"
          />
        </div>
      </div>

      <div class="self-center w-6">
        <chevron-right-icon size="1.5x" class="button" />
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

<style scoped>
.application-card {
  @apply shadow-md;
  background-color: #262626;
}

.card-header {
  @apply text-white;
  @apply text-sm;
  @apply font-normal;
  background-color: #3a3d40;
}

.visualization-card {
  @apply bg-gray-900;
  @apply border;
  @apply border-gray-600;
  width: 25%;
}

.visualization-title {
  @apply text-xs;
}

.button {
  color: #4a4b4c;
}

.button:hover {
  @apply text-gray-400;
}

.button:active {
  @apply text-gray-300;
}
</style>
