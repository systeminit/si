<template>
  <div class="flex flex-col justify-center w-full bg-gray-800">
    <div class="pl-3 text-white bg-gray-700">
      {{ application.name }}
    </div>

    <div class="flex w-full my-3 ">
      <div class="flex justify-between w-full">
        <div class="visualization-title ">
          Activity
          <!-- <ActivityVisualization class="mx-2 mb-2" /> -->
        </div>

        <div class="visualization-title ">
          Services
          <!-- 
          <ServicesVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
            inEditor="false"
          /> 
          -->
        </div>

        <div class="visualization-title ">
          Systems
          <!--
          <SystemsVisualization
            class="mx-2 mb-2"
            :applicationId="application.id"
          />
          -->
        </div>

        <div class="visualization-title ">
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

        <div class="self-center w-6">
          <chevron-right-icon size="1.5x" class="button" />
        </div>
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
.visualization-title {
  @apply block;
  @apply h-full;
  @apply pt-2;
  @apply pl-3;
  @apply mx-3;
  @apply text-xs;
  @apply bg-gray-900;
  @apply border;
  @apply border-gray-600;
  width: 25%;
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
