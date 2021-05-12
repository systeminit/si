<template>
  <div>
    <div class="connection-title">
      <div class="px-4 py-1 text-sm font-normal">{{ title }}</div>
    </div>

    <div class="mx-4 mt-4 ml-4" v-show="connections">
      <div
        class="flex flex-row items-center justify-between mt-2"
        v-for="(connection, index) in connections"
        :key="index"
      >
        <div
          class="flex flex-row items-center justify-between w-full"
          v-if="!reversed"
        >
          <div
            class="flex flex-row items-center border border-gray-400 connection-details"
          >
            <div class="flex flex-col justify-end px-2 py-2">
              <div class="flex justify-end connection-node-name">
                {{ connection.source.nodeName }}
              </div>
              <div class="flex justify-end connection-node-type">
                ({{ connection.source.nodeType }})
              </div>

              <div class="mt-1 text-xs font-normal">
                <div class="flex justify-center connection-socket-name">
                  {{ connection.source.socketName }}
                </div>
              </div>
            </div>
          </div>

          <ArrowRightIcon size="1x" class="mx-4" />

          <div class="flex flex-row items-center text-xs font-normal">
            <div class="mr-2 connection-socket" />
            {{ connection.destination.socketName }}
          </div>
        </div>

        <div
          class="flex flex-row items-center justify-between w-full"
          v-if="reversed"
        >
          <div
            class="flex flex-row items-center text-xs font-normal text-gray-300"
          >
            {{ connection.source.socketName }}
            <div class="ml-2 connection-socket" />
          </div>

          <ArrowRightIcon size="1x" class="mx-4" />

          <div
            class="flex flex-row items-center justify-end border border-gray-400 connection-details"
          >
            <div class="flex flex-col justify-end px-2 py-2">
              <div class="flex justify-end connection-node-name">
                {{ connection.destination.nodeName }}
              </div>
              <div class="flex justify-end connection-node-type">
                ({{ connection.destination.nodeType }})
              </div>

              <div class="mt-1 text-xs font-normal">
                <div class="flex justify-center connection-socket-name">
                  {{ connection.destination.socketName }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="w-8 ml-1">
          <button
            v-show="enableConnectionDelete"
            class="focus:outline-none delete-connection-button"
            @click="disconnect(connection.edge.id)"
          >
            <TrashIcon size="0.75x" class="ml-2" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import Vue, { PropType } from "vue";
import { ArrowRightIcon, TrashIcon } from "vue-feather-icons";
import { Connection } from "@/api/sdf/model/connection";

export default Vue.extend({
  name: "ConnectionSection",
  components: {
    ArrowRightIcon,
    TrashIcon,
  },
  props: {
    connections: {
      type: Array as PropType<Connection[]>,
      required: true,
      default: null,
    },
    reversed: {
      type: Boolean,
      default: false,
    },
    enableConnectionDelete: {
      type: Boolean,
      default: false,
    },
    title: String,
  },
  methods: {
    disconnect(edgeId: string) {
      this.$emit("delete-connection", edgeId);
    },
  },
});
</script>

<style scoped>
.connection-title {
  background-color: #292c2d;
  color: #e9f2fe;
}
.connection-socket {
  display: block;
  height: 12px;
  width: 12px;
  background-color: #282e30;
  border-radius: 50%;
  border-width: 1px;
  border-color: #008ed2;
}

.connection-details {
  font-weight: 400;
  font-size: 11px;
  line-height: 1rem;
  background-color: #282e30;
  border-color: #323b3d;
}

.connection-node-type {
  color: #aec6d3;
}

.connection-node-name {
  color: #b7d3ae;
}

.connection-socket-name {
  color: #d3c9ae;
}

.delete-connection-button {
  color: #f43f5e;
}
</style>
