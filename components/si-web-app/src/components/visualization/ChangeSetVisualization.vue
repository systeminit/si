<template>
  <div class="flex flex-col">
    <div class="ml-1 text-sm font-bold text-gray-400">
      changes
    </div>

    <div class="flex pl-1">
      <div class="flex justify-end w-1/4 ml-1 text-xs font-bold text-gray-400">
        open:
      </div>
      <div class="w-3/4">
        <div
          class="flex ml-1 text-xs font-normal text-gray-400"
          data-cy="change-set-visualization-open-count"
        >
          {{ openCount }}
          <alert-circle-icon
            v-if="openCount"
            size="0.75x"
            class="ml-1 self-center text-orange-600"
          />
        </div>
      </div>
    </div>
    <div class="flex pl-1">
      <div class="flex justify-end w-1/4 ml-1 text-xs font-bold text-gray-400">
        closed:
      </div>
      <div
        class="flex w-3/4 ml-1 text-xs font-normal text-gray-400"
        data-cy="change-set-visualization-closed-count"
      >
        {{ closedCount }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { AlertCircleIcon } from "vue-feather-icons";

export default Vue.extend({
  name: "ChangeSetVisualization",
  props: {
    applicationId: String,
  },
  components: {
    AlertCircleIcon,
  },
  computed: {
    openCount(): number {
      return this.$store.getters["changeSet/count"]({
        forId: this.applicationId,
        status: "OPEN",
      });
    },
    closedCount(): number {
      return this.$store.getters["changeSet/count"]({
        forId: this.applicationId,
        status: "CLOSED",
      });
    },
  },
});
</script>
