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
          {{ count.open }}
          <alert-circle-icon
            v-if="count.open"
            size="0.75x"
            class="self-center ml-1 text-orange-600"
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
        {{ count.closed }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState } from "vuex";
import { AlertCircleIcon } from "vue-feather-icons";

import { RootStore } from "@/store";

export default Vue.extend({
  name: "ChangeSetVisualization",
  props: {
    applicationId: String,
  },
  components: {
    AlertCircleIcon,
  },
  computed: {
    ...mapState({
      count(state: RootStore) {
        if (state.application.changeSetCounts[this.applicationId]) {
          return state.application.changeSetCounts[this.applicationId];
        } else {
          return { open: "loading", count: "loading" };
        }
      },
    }),
  },
});
</script>
