<template>
  <div class="flex flex-col w-full">
    <div class="flex flex-col">
      <div class="flex flex-row text-gray-400">
        <div class="flex text-xs font-medium">
          open:
        </div>

        <div class="flex ml-1 text-xs" data-testid="changeSetCountsOpen">
          {{ changeSetCounts.open }}
          <alert-circle-icon
            v-if="changeSetCounts.open"
            size="0.75x"
            class="self-center ml-1 text-orange-600"
          />
        </div>
      </div>

      <div class="flex flex-row text-gray-400">
        <div class="flex text-xs font-medium">
          closed:
        </div>
        <div
          class="flex ml-1 text-xs font-normal"
          data-testid="changeSetCountsClosed"
        >
          {{ changeSetCounts.closed }}
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { IApplicationListEntry } from "@/store/modules/application";

export default Vue.extend({
  name: "ChangeSetCounts",
  props: {
    changeSetCounts: {
      type: Object as PropType<IApplicationListEntry["changeSetCounts"]>,
    },
  },
});
</script>
