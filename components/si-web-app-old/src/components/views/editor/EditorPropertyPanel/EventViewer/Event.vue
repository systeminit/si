<template>
  <div :id="event.id" class="mx-4 mt-3 event">
    <div class="event-summary">
      <div class="flex flex-col">
        <div
          class="flex justify-between event-summary-header pr-2 pl-2 py-1 text-gray-400 text-xs"
        >
          <div class="flex justify-between ">
            <div class="">
              <Tooltip>
                {{ event.localTime() }}
                <template v-slot:tooltip>
                  <div class="flex flex-col text-gray-500">
                    {{ event.relativeToNow() }}
                  </div>
                </template>
              </Tooltip>
            </div>

            <div class="pl-1 pr-1">
              |
            </div>

            <div class="">
              {{ event.name }}
            </div>
          </div>

          <div
            class="text-xs text-right"
            :class="{
              'event-succeeded': eventSucceeded,
              'event-failed': eventFailed,
              'event-running': eventRunning,
              'event-unknown': eventUnknown,
            }"
          >
            {{ event.status }}
          </div>
        </div>

        <div
          class="flex justify-between w-full event-summary-body pr-2 pl-2 py-1 text-gray-400 text-xs"
        >
          <div class="flex flex-col mr-4">
            <div class="flex items-start">
              <alert-triangle-icon
                size="1.0x"
                class="text-red-500"
                v-if="eventFailed"
              />
              <div class="w-3" v-if="!eventFailed"></div>
              <div class="pl-2 pr-1">
                Root event:
              </div>

              <div class="">
                {{ event.name }}
              </div>
            </div>

            <div class="pl-5">
              {{ event.localTime() }}
            </div>
          </div>

          <div class="flex flex-col">
            <div class="flex items-start">
              <div class="pr-1">
                Status:
              </div>
              <div class="justify-center">
                Failed
              </div>
            </div>

            <div class="flex items-start">
              <div class="pr-1">
                Triggered by:
              </div>
              <div class="justify-center">
                {{ event.owner }}
                <!-- Identify Root event -->
              </div>
            </div>
          </div>

          <div class="flex items-end text-right text-white">
            <button
              class="mr-1 focus:outline-none"
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

    <div
      class="event-details"
      :class="{
        hidden: !this.showEventDetails,
        display: this.showEventDetails,
      }"
    >
      <!--       <div
        v-for="parent in parents"
        :key="parent.id"
        class="ml-2 text-xs text-right text-gray-400"
      >
        via {{ parent.message }} ({{ parent.status }})
      </div> -->
      <div v-for="log in eventLogs" :key="log.id" class="event-operation">
        <EventOperation :eventLog="log" />
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";

import {
  ChevronDownIcon,
  ChevronRightIcon,
  AlertTriangleIcon,
} from "vue-feather-icons";

import EventOperation from "./EventOperation.vue";
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
    AlertTriangleIcon,
    EventOperation,
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

.event-summary {
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

.event-summary-header {
  background-color: #2a2c2d;
}

.event-summary-body {
  background-color: #313536;
}

.event-operation:nth-child(even) {
  background-color: #121314;
}

.event-operation:nth-child(odd) {
  background-color: #181a1b;
}
</style>
