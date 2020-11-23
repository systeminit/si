<template>
  <div :id="event.id" class="mx-4 mt-3">
    <div class="event-summary-bar">
      <div class="">
        <div class="flex justify-between">
          <div class="flex flex-col m-2">
            <div class="justify-center text-white text-md">
              {{ event.name }}
            </div>
            <div class="justify-center text-xs text-white">
              {{ event.owner }}
            </div>
            <div class="justify-center text-xs text-white">
              <Tooltip>
                {{ event.localTime() }}
                <template v-slot:tooltip>
                  <div class="flex-col text-sm text-gray-400 felx">
                    {{ event.relativeToNow() }}
                  </div>
                </template>
              </Tooltip>
            </div>
          </div>

          <div class="flex flex-col justify-between">
            <div
              class="pt-1 mr-2 text-xs text-right text-white"
              :class="{
                'event-succeeded': eventSucceeded,
                'event-failed': eventFailed,
                'event-running': eventRunning,
                'event-unknown': eventUnknown,
              }"
            >
              {{ event.status }}
            </div>

            <div class="text-right text-white">
              <button
                class="justify-end mr-1 focus:outline-none"
                :class="{
                  hidden: this.showEventDetails,
                  display: !this.showEventDetails,
                }"
                @click="expandEventDetails()"
              >
                <chevron-right-icon size="1.1x" class="custom-class" />
              </button>

              <button
                class="justify-end mr-1 focus:outline-none"
                :class="{
                  hidden: !this.showEventDetails,
                  display: this.showEventDetails,
                }"
                @click="collapseEventDetails()"
              >
                <chevron-down-icon size="1.1x" class="custom-class" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      class="event-data"
      :class="{
        hidden: !this.showEventDetails,
        display: this.showEventDetails,
      }"
    >
      <div
        v-for="parent in parents"
        :key="parent.id"
        class="ml-2 text-xs text-right text-gray-400"
      >
        via {{ parent.message }} ({{ parent.status }})
      </div>
      <div v-for="log in eventLogs" :key="log.id" class="event-msg">
        <EventLogElem :eventLog="log" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import { ChevronDownIcon, ChevronRightIcon } from "vue-feather-icons";

import EventLogElem from "./EventLog.vue";
import Tooltip from "@/components/ui/Tooltip.vue";
import { Event, EventKind, EventStatus } from "@/api/sdf/model/event";
import { User } from "@/api/sdf/model/user";
import { EventLog } from "@/api/sdf/model/eventLog";

export default Vue.extend({
  name: "Event",
  props: {
    event: {
      type: Object as PropType<Event>,
    },
  },
  components: {
    ChevronDownIcon,
    ChevronRightIcon,
    EventLogElem,
    Tooltip,
  },
  data() {
    return {
      showEventDetails: false,
    };
  },
  computed: {
    eventRunning(): boolean {
      if (this.event.status == EventStatus.Running) {
        return true;
      } else {
        return false;
      }
    },
    eventUnknown(): boolean {
      if (this.event.status == EventStatus.Unknown) {
        return true;
      } else {
        return false;
      }
    },
    eventSucceeded(): boolean {
      if (this.event.status == EventStatus.Success) {
        return true;
      } else {
        return false;
      }
    },
    eventFailed(): boolean {
      if (this.event.status == EventStatus.Error) {
        return true;
      } else {
        return false;
      }
    },
    parents(): EventLog[] {
      if (this.$store.state.event.parents[this.event.id]) {
        return this.$store.state.event.parents[this.event.id];
      } else {
        return [];
      }
    },
    eventLogs(): EventLog[] {
      if (this.$store.state.event.logs[this.event.id]) {
        return this.$store.state.event.logs[this.event.id];
      } else {
        return [];
      }
    },
  },
  methods: {
    async expandEventDetails(): Promise<void> {
      await this.$store.dispatch("event/loadParents", {
        eventId: this.event.id,
      });
      await this.$store.dispatch("event/loadLogs", { eventId: this.event.id });
      this.showEventDetails = true;
    },
    collapseEventDetails(): void {
      this.showEventDetails = false;
    },
  },
});
</script>

<style type="text/css" scoped>
.content {
  background-color: #2a2c2d;
}

.event-summary-bar {
  background-color: #2a2c2d;
}

.event-running {
  color: #5555ff;
}

.event-unknown {
  color: #ffbbff;
}

.event-succeeded {
  color: #88ff4c;
}

.event-failed {
  color: #ff624c;
}

.event-msg:nth-child(even) {
  background-color: #121314;
}

.event-msg:nth-child(odd) {
  background-color: #181a1b;
}
</style>
