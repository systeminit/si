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
        <div class="w-full text-xs event-list-row">
          <div
            v-for="eventItem in events"
            :key="eventItem.id"
            class="flex flex-col text-xs text-gray-400 event-list-row"
          >
            <div
              class="flex flex-row items-center"
              @click="toggleEventExpansion(eventItem.id)"
            >
              <div class="ml-8">
                <ChevronDownIcon
                  size="1.0x"
                  v-if="isEventExpanded(eventItem.id)"
                />
                <ChevronRightIcon size="1.0x" v-else />
              </div>
              <div class="ml-2">
                <span class="text-gray-500">
                  {{ eventItem.event.status }}
                </span>
                <span class="text-gray-600">
                  {{ eventItem.event.relativeToNow() }}
                </span>
                <span class="text-gray-400">
                  {{ eventItem.event.name }}: {{ eventItem.event.message }}
                </span>
              </div>
            </div>
            <div class="ml-16 mr-10" v-if="isEventExpanded(eventItem.id)">
              <div class="flex-row">
                <div
                  class="flex overflow-auto text-xs text-gray-700"
                  v-for="eventLog in eventItem.logs"
                  :key="eventLog.id"
                >
                  <div class="w-10 ml-2">{{ eventLog.level }}</div>
                  <div class="ml-2">
                    {{ eventLog.localTime() }}
                  </div>
                  <div class="ml-2">{{ eventLog.message }}</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="flex items-center w-full h-6 text-xs event-bar">
      <div class="flex items-center" v-if="eventLatest">
        <div class="ml-6 text-xs text-gray-100">event</div>
        <div class="ml-4 text-gray-400">
          <span class="text-gray-500">
            {{ eventLatest.event.status }}
          </span>
          <span class="text-gray-600">
            {{ eventLatest.event.relativeToNow() }}
          </span>
          <span class="text-gray-400">
            {{ eventLatest.event.name }}: {{ eventLatest.event.message }}
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
import {
  MaximizeIcon,
  MinimizeIcon,
  ChevronRightIcon,
  ChevronDownIcon,
} from "vue-feather-icons";
import { mapState, mapGetters } from "vuex";
import { camelCase } from "change-case";
import { RootStore } from "@/store";
import { Event } from "@/api/sdf/model/event";

interface Data {
  expanded: boolean;
  interval: number | null | undefined;
  eventExpanded: {
    [key: string]: boolean;
  };
}

export default Vue.extend({
  name: "EventBar",
  components: {
    MaximizeIcon,
    MinimizeIcon,
    ChevronRightIcon,
    ChevronDownIcon,
  },
  data(): Data {
    return {
      expanded: false,
      interval: null,
      eventExpanded: {},
    };
  },
  methods: {
    toggleExpand() {
      this.expanded = !this.expanded;
    },
    isEventExpanded(eventLogId: string): boolean {
      if (this.eventExpanded[eventLogId]) {
        return true;
      } else {
        return false;
      }
    },
    toggleEventExpansion(eventLogId: string): void {
      if (this.eventExpanded[eventLogId]) {
        Vue.set(this.eventExpanded, eventLogId, false);
      } else {
        Vue.set(this.eventExpanded, eventLogId, true);
      }
    },
  },
  computed: {
    eventLatest(): Event | undefined {
      if (this.events.length > 0) {
        return this.events[0].event;
      } else {
        return undefined;
      }
    },
    ...mapState({
      events: (state: any): RootStore["editor"]["eventBar"] => {
        return state.editor.eventBar;
      },
    }),
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
