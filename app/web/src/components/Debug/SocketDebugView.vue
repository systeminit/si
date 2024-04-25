<template>
  <div class="overflow-hidden my-xs p-xs border-opacity-10 border-l-2">
    <dl class="flex flex-col gap-xs">
      <DebugViewItem title="Attribute Value Id" :data="data.attributeValueId" />
      <DebugViewItem :data="data.kind ?? 'any'" title="Type" />
      <DebugViewItem
        :data="`${data.funcName} ${data.funcId}`"
        title="Set By Function"
      />
      <DebugViewItem title="Input" :data="data.funcArgs ?? 'NULL'" />
      <DebugViewItem title="Input sources">
        <template #data>
          <ul v-if="data.argSources && Object.keys(data.argSources).length">
            <li v-for="[k, v] in Object.entries(data.argSources)" :key="k">
              <strong>{{ k }}</strong>
              : {{ v ?? "?" }}
            </li>
          </ul>
          <p v-else>No input sources</p>
        </template>
      </DebugViewItem>
      <DebugViewItem title="Value" :data="data.value ?? 'NULL'" />
      <DebugViewItem title="Prototype Id" :data="data.prototypeId" />
      <DebugViewItem title="Socket Id" :data="data.socketId" />
      <DebugViewItem title="Connection Annotations">
        <template #data>
          <ul
            v-if="
              data.connectionAnnotations && data.connectionAnnotations.length
            "
          >
            <li
              v-for="connection in data.connectionAnnotations"
              :key="connection"
              :data="connection"
            >
              {{ connection }}
            </li>
          </ul>
          <p v-else>No input sources</p>
        </template>
      </DebugViewItem>
      <DebugViewItem title="Inferred Connection(s)">
        <template #data>
          <ul
            v-if="data.inferredConnections && data.inferredConnections.length"
          >
            <li
              v-for="connection in data.inferredConnections"
              :key="connection"
              :data="connection"
            >
              {{ connection }}
            </li>
          </ul>
          <p v-else>No input sources</p>
        </template>
      </DebugViewItem>
      <DebugViewItem
        title="Materialized View"
        :data="data.materializedView ?? 'NULL'"
      />
    </dl>
  </div>
</template>

<script setup lang="ts">
import { SocketDebugView } from "@/store/components.store";
import DebugViewItem from "./DebugViewItem.vue";

defineProps<{ data: SocketDebugView }>();
</script>
