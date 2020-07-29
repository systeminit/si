<template>
  <div class="event-bar">
    <div v-if="expanded" class="flex-col w-full h-48">
      <div class="flex items-center h-6">
        <div class="ml-6 text-base text-gray-100">
          Events
        </div>
        <div class="flex justify-end flex-grow mr-2 text-gray-100">
          <button @click="toggleExpand">
            <MinimizeIcon size="1.0x" />
          </button>
        </div>
      </div>

      <div class="flex-col h-40 overflow-y-auto event-list">
        <div class="items-center w-full h-3 text-xs event-list-row" />

        <div
          v-for="eventLog in eventLogs"
          :key="eventLog.id"
          class="flex items-center h-5 text-xs text-gray-400 event-list-row"
        >
          <div class="ml-8">
            <span class="text-gray-500">
              {{ eventLog.level }}
            </span>
            <span class="text-gray-600">
              {{ eventLog.timestamp }}
            </span>
            <span class="text-gray-400">
              {{ eventLog.message }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="flex items-center w-full h-6 text-xs event-bar">
      <div class="flex items-center">
        <div class="ml-6 text-xs text-gray-100">event</div>
        <div class="ml-4 text-gray-400">
          <span class="text-gray-500">
            {{ eventLogLatest.level }}
          </span>
          <span class="text-gray-600">
            {{ eventLogLatest.timestamp }}
          </span>
          <span class="text-gray-400">
            {{ eventLogLatest.message }}
          </span>
        </div>
      </div>
      <div class="flex justify-end flex-grow mr-2 text-gray-100">
        <button @click="toggleExpand">
          <MaximizeIcon size="1.0x" />
        </button>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { MaximizeIcon, MinimizeIcon } from "vue-feather-icons";
import { mapState, mapGetters } from "vuex";
import { camelCase } from "change-case";
import { RootStore } from "@/store";

interface Data {
  expanded: boolean;
  interval: number | null | undefined;
}

export default Vue.extend({
  name: "EventBar",
  components: {
    MaximizeIcon,
    MinimizeIcon,
  },
  data(): Data {
    return {
      expanded: false,
      interval: null,
    };
  },
  methods: {
    toggleExpand() {
      this.expanded = !this.expanded;
    },
  },
  computed: {
    ...mapGetters({
      eventLogLatest: "eventLog/latest",
    }),
    ...mapState({
      eventLogs: (state: any): RootStore["eventLog"]["eventLogs"] =>
        state.eventLog.eventLogs,
    }),
  },
  mounted(): void {
    const handle = setInterval(() => {
      this.$store.dispatch("eventLog/load");
    }, 5000);
    // @ts-ignore
    this.interval = handle;
  },
  beforeDestroy(): void {
    if (this.interval != undefined) {
      // @ts-ignore
      clearInterval(this.interval);
    }
  },
});
</script>

<style>
.event-bar {
  background-color: #1f2324;
}

.event-bar-header {
  background-color: #2c2e30;
}

.event-list div.event-list-row:nth-child(even) {
  background-color: #202324;
}

.event-list div.event-list-row:nth-child(odd) {
  background-color: #181a1b;
}

.mode-edit {
  color: #d9a35e;
}
.mode-view {
  color: #3091c1;
}
</style>
