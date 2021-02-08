<template>
  <div class="flex flex-col">
    <div class="ml-1 text-sm font-bold text-gray-400">changes</div>

    <div class="flex pl-1">
      <div class="flex justify-end w-1/4 ml-1 text-xs font-bold text-gray-400">
        open:
      </div>
      <div class="w-3/4">
        <div
          class="flex ml-1 text-xs font-normal text-gray-400"
          data-testid="changeSetCountsOpen"
        >
          {{ changeSetCounts.open }}
          <alert-circle-icon
            v-if="changeSetCounts.open"
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
        data-testid="changeSetCountsClosed"
      >
        {{ changeSetCounts.closed }}
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
