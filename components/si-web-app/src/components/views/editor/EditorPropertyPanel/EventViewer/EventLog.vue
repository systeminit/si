<template>
  <div class="flex-row">
    <div class="flex overflow-auto text-xs text-gray-700">
      <div class="w-10 ml-2">{{ eventLog.level }}</div>
      <div class="ml-2">
        <Tooltip>
          {{ eventLog.localTime() }}
          <template v-slot:tooltip>
            <div class="flex-col text-sm text-gray-400 felx">
              {{ eventLog.relativeToNow() }}
            </div>
          </template>
        </Tooltip>
      </div>
      <div class="ml-2">{{ eventLog.message }}</div>
      <div class="flex-grow text-right text-white">
        <button
          class="justify-end mr-1 focus:outline-none"
          v-if="!showOutputLines"
          @click="expandOutputLines()"
        >
          <chevron-right-icon size="1.1x" class="custom-class" />
        </button>

        <button
          class="justify-end mr-1 focus:outline-none"
          v-if="showOutputLines"
          @click="collapseOutputLines()"
        >
          <chevron-down-icon size="1.1x" class="custom-class" />
        </button>
      </div>
    </div>
    <div class="flex-row" v-if="showOutputLines">
      <div class="flex overflow-auto text-xs text-gray-300 h-60">
        <pre class="whitespace-pre-wrap w-60">
          <code class="whitespace-pre-wrap">
{{ outputLines.map(obj => obj.line).join("\n") }}
          </code>
        </pre>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { mapState, mapGetters } from "vuex";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";
import _ from "lodash";

import { EventLog } from "@/api/sdf/model/eventLog";
import { OutputLine } from "@/api/sdf/model/outputLine";
import Tooltip from "@/components/ui/Tooltip.vue";

interface Data {
  showOutputLines: boolean;
}

export default Vue.extend({
  name: "EventLog",
  props: {
    eventLog: {
      type: Object as PropType<EventLog>,
    },
  },
  components: {
    ChevronRightIcon,
    ChevronDownIcon,
    Tooltip,
  },
  data(): Data {
    return {
      showOutputLines: false,
    };
  },
  methods: {
    async expandOutputLines(): Promise<void> {
      await this.$store.dispatch("event/loadOutputLines", {
        eventLogId: this.eventLog.id,
      });
      this.showOutputLines = true;
    },
    collapseOutputLines(): void {
      this.showOutputLines = false;
    },
  },
  computed: {
    ...mapState({
      output: (state: any): Record<string, OutputLine[]> => state.event.output,
    }),
    outputLines(): OutputLine[] {
      if (this.output[this.eventLog.id]) {
        const lines = this.output[this.eventLog.id];
        return _.filter(lines, ["stream", "all"]);
      } else {
        return [];
      }
    },
  },
});
</script>
