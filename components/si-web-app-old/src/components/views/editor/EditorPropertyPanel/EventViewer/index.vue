<template>
  <div id="event-viewer" class="h-full">
    <div
      id="event-viewer-menu-bar"
      class="flex h-12 pl-6 pr-6 text-white menu-bar"
    >
      <div class="self-center w-3/5 text-lg text-white">Events</div>

      <div class="flex self-center justify-end w-2/5">
        <div class="flex self-center justify-center w-2/5">
          <button
            class="focus:outline-none"
            :class="{ 'button-filter-succeeded': this.filterSucceeded }"
            @click="setFilterSucceeded()"
          >
            <check-circle-icon size="1.1x" class="custom-class" />
          </button>
          <button
            class="pl-2 focus:outline-none"
            :class="{ 'button-filter-failed': this.filterFailed }"
            @click="setFilterFailed()"
          >
            <alert-circle-icon size="1.1x" class="custom-class" />
          </button>
        </div>

        <div class="flex self-center justify-end w-1/5">
          <button
            class="focus:outline-none"
            :class="{
              'button-filter-filterSuccessorsEvents': this
                .filterSuccessorsEvents,
            }"
            @click="setFilterSuccessorsEvents()"
          >
            <cast-icon size="1.1x" class="custom-class" />
          </button>
        </div>
      </div>
    </div>

    <div id="event-viewer-content" class="h-full mb-20 overflow-auto">
      <div v-for="event in eventList" :key="event.id">
        <!-- <div v-if="event.statusCode === 1 && filterSucceeded"> -->
        <Event :event="event" />
        <!-- </div> -->

        <div v-if="event.statusCode === 0 && filterFailed">
          <Event :event="event" />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue from "vue";
import { mapState, mapGetters } from "vuex";

import { CheckCircleIcon, AlertCircleIcon, CastIcon } from "vue-feather-icons";

import { EditorStore } from "@/store/modules/editor";
import { EventStore } from "@/store/modules/event";
import Event from "./Event.vue";

export default Vue.extend({
  name: "EventViewer",
  components: {
    CheckCircleIcon,
    AlertCircleIcon,
    CastIcon,
    Event,
  },
  props: {
    selectedNode: Object,
  },
  data() {
    const editObject = this.$store.state.editor.editObject;
    if (editObject?.id) {
      this.$store.dispatch("event/setContext", {
        context: [editObject?.id],
      });
    } else {
      this.$store.dispatch("event/setContext", {
        context: [],
      });
    }

    return {
      filterSucceeded: true,
      filterFailed: true,
      filterSuccessorsEvents: false,
    };
  },
  methods: {
    setFilterSucceeded(): void {
      this.filterSucceeded = !this.filterSucceeded;
    },
    setFilterFailed(): void {
      this.filterFailed = !this.filterFailed;
    },
    setFilterSuccessorsEvents(): void {
      this.filterSuccessorsEvents = !this.filterSuccessorsEvents;
    },
    maximizePanel(): void {
      this.$emit("maximizePanelMsg", {
        panel: {
          id: "property",
        },
      });
    },
  },
  computed: {
    ...mapState({
      eventList: (state: any): EventStore["list"] => state.event.list,
      editObject: (state: any): EditorStore["editObject"] =>
        state.editor.editObject,
    }),
  },
  watch: {
    async editObject(value): Promise<void> {
      if (this.editObject?.id) {
        this.$store.dispatch("event/setContext", {
          context: [this.editObject.id],
        });
      } else {
        this.$store.dispatch("event/setContext", {
          context: [],
        });
      }
    },
  },
});
</script>

<style type="text/css" scoped>
.menu-bar {
  background-color: #2a2c2d;
}

.content {
  background-color: #2a2c2d;
}

.button-filter-succeeded {
  color: #88ff4c;
}

.button-filter-failed {
  color: #ff624c;
}

.button-filter-filterSuccessorsEvents {
  color: #ffd44c;
}
</style>
