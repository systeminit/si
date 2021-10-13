<template>
  <div class="flex flex-col justify-center w-full application-card">
    <div class="py-1 pl-2 card-header">
      {{ application.name }}
    </div>

    <div class="flex w-full h-full ">
      <div class="w-1/5 py-2 mx-2 ">
        <ActivitySummary
          height="40px"
          :applicationId="applicationEntry.application.id"
        />
      </div>

      <div class="w-1/5 py-2 mx-2 ">
        <ServicesSummary
          :applicationId="applicationEntry.application.id"
          :showButton="false"
        />
      </div>

      <div class="w-1/5 py-2 mx-2 ">
        <ComputingResourceSummary
          :applicationId="applicationEntry.application.id"
          :showButton="false"
        />
      </div>

      <div class="w-1/5 py-2 mx-2 ">
        <ProviderSummary
          :applicationId="applicationEntry.application.id"
          :showButton="false"
        />
      </div>

      <div class="w-1/5 py-2 mx-2 ">
        <ChangesSummary
          :applicationId="applicationEntry.application.id"
          :showSelectedChangesetData="false"
        />
      </div>

      <div class="self-center w-6">
        <chevron-right-icon size="1.5x" class="button" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { RawLocation } from "vue-router";

import { ChevronRightIcon } from "vue-feather-icons";

import ActivitySummary from "@/molecules/ActivitySummary.vue";
import ServicesSummary from "@/molecules/ServicesSummary.vue";
import ComputingResourceSummary from "@/molecules/ComputingResourceSummary.vue";
import ChangesSummary from "@/molecules/ChangesSummary.vue";
import ProviderSummary from "@/molecules/ProviderSummary.vue";
import { IApplicationCreateReplySuccess } from "@/api/sdf/dal/applicationDal";

export default Vue.extend({
  name: "ApplicationDetailCard",
  props: {
    applicationEntry: {
      type: Object as PropType<IApplicationCreateReplySuccess>,
    },
    linkTo: {
      type: Object as PropType<RawLocation>,
    },
  },
  components: {
    ChevronRightIcon,
    ActivitySummary,
    ServicesSummary,
    ComputingResourceSummary,
    ChangesSummary,
    ProviderSummary,
  },
  computed: {
    application(): IApplicationCreateReplySuccess["application"] {
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
