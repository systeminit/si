<template>
  <div
    v-if="incoming.length > 0 || outgoing.length > 0"
    :class="
      clsx(
        inMap && 'border rounded pb-sm',
        themeClasses(
          'border-neutral-400 bg-white',
          'border-neutral-600 bg-neutral-900',
        ),
      )
    "
  >
    <template v-if="incoming.length">
      <ConnectionLayout
        label="Incoming"
        :connections="incoming"
        :highlightedPath="highlightedPath"
      />
    </template>
    <template v-if="outgoing.length">
      <ConnectionLayout label="Outgoing" :connections="outgoing" />
    </template>
  </div>
  <EmptyState
    v-else-if="!inMap"
    icon="output-connection"
    text="No connections yet"
  />
</template>

<script setup lang="ts">
import { computed, ref } from "vue";
import clsx from "clsx";
import { themeClasses } from "@si/vue-lib/design-system";
import {
  IncomingConnections,
  ComponentInList,
  BifrostComponent,
} from "@/workers/types/entity_kind_types";
import ConnectionLayout from "./layout_components/ConnectionLayout.vue";
import { useConnections } from "./logic_composables/connections";
import EmptyState from "./EmptyState.vue";

const props = defineProps<{
  component: ComponentInList | BifrostComponent;
  connections?: IncomingConnections;
  inMap?: boolean;
}>();

const connectionsGetter = useConnections();
const connections = computed(
  () => connectionsGetter(props.component.id, props.connections).value,
);
const incoming = computed(() => connections.value.incoming);
const outgoing = computed(() => connections.value.outgoing);

const highlightedPath = ref("");
const highlight = (selfPath: string) => {
  highlightedPath.value = selfPath;
};

defineExpose({
  highlight,
});
</script>
