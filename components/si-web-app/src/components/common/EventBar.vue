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
            v-for="(eventLog, index) of eventLogs"
            :key="index"
            class="flex flex-col text-xs text-gray-400 event-list-row"
          >
            <div
              class="flex flex-row items-center"
              @click="toggleEventExpansion(eventLog.id)"
            >
              <div class="ml-8">
                <ChevronDownIcon
                  size="1.0x"
                  v-if="isEventExpanded(eventLog.id)"
                />
                <ChevronRightIcon size="1.0x" v-else />
              </div>
              <div class="ml-2">
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
            <div class="ml-16 mr-10" v-if="isEventExpanded(eventLog.id)">
              <div
                v-if="eventLog.payload.siStorable.typeName == 'entity'"
                class="flex-col"
              >
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      name:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.name }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      type:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.siStorable.typeName }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    event:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.name }}
                    </div>
                  </div>
                </div>
              </div>
              <div
                v-else-if="eventLog.payload.kind == 'change_set'"
                class="flex-col"
              >
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      name:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.name }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      type:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.siStorable.typeName }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    event:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.name }}
                    </div>
                  </div>
                </div>
              </div>
              <div
                v-else-if="
                  eventLog.payload.kind == 'change_set_entry' &&
                    eventLog.payload.data.actionName
                "
                class="flex-col"
              >
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>action:</div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.actionName }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      type:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.siStorable.typeName }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    event:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.name }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    entity name:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.inputEntity.name }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    entity type:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{
                        eventLog.payload.data.inputEntity.siStorable.typeName
                      }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    success:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.success }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    stdout:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      <pre>{{
                        eventLog.payload.data.outputLines.join("\n")
                      }}</pre>
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    stderr:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      <pre>{{
                        eventLog.payload.data.errorLines.join("\n")
                      }}</pre>
                    </div>
                  </div>
                </div>
              </div>
              <div
                v-else-if="eventLog.payload.kind == 'change_set_entry'"
                class="flex-col"
              >
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      action:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      edited
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    <div>
                      type:
                    </div>
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.siStorable.typeName }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    event:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.name }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    entity name:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.name }}
                    </div>
                  </div>
                </div>
                <div class="flex flex-row w-full">
                  <div class="flex justify-end w-1/12 mr-2 text-white">
                    entity type:
                  </div>
                  <div class="flex justify-start w-11/12">
                    <div>
                      {{ eventLog.payload.data.siStorable.typeName }}
                    </div>
                  </div>
                </div>
              </div>

              <div v-else>
                <pre>
                <code>
{{ JSON.stringify(eventLog, null, 2) }}
                </code>
                </pre>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="flex items-center w-full h-6 text-xs event-bar">
      <div class="flex items-center" v-if="eventLogLatest">
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
import {
  MaximizeIcon,
  MinimizeIcon,
  ChevronRightIcon,
  ChevronDownIcon,
} from "vue-feather-icons";
import { mapState, mapGetters } from "vuex";
import { camelCase } from "change-case";
import { RootStore } from "@/store";
import { EventLog } from "@/api/sdf/model/eventLog";

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
    eventLogLatest(): EventLog | undefined {
      if (this.eventLogs.length > 0) {
        return this.eventLogs[0];
      } else {
        return undefined;
      }
    },
    ...mapState({
      eventLogs: (state: any): RootStore["editor"]["eventLogs"] => {
        return state.editor.eventLogs;
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
