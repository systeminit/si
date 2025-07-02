<template>
  <div
    v-if="incoming.length > 0 || outgoing.length > 0"
    :class="
      clsx(
        'border rounded pb-sm',
        themeClasses(
          'border-neutral-400 bg-white',
          'border-neutral-600 bg-neutral-900',
        ),
      )
    "
  >
    <template v-if="incoming.length">
      <ConnectionLayout label="Inputs" :connections="incoming" />
    </template>
    <template v-if="outgoing.length">
      <ConnectionLayout label="Outputs" :connections="outgoing" />
    </template>
  </div>
  <!-- TODO(Wendy) - separate empty states for input and output connections? -->
  <EmptyState
    v-else-if="!noEmptyState"
    icon="input-connection"
    text="No connections yet"
  />
</template>

<script setup lang="ts">
import { computed } from "vue";
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
  noEmptyState?: boolean;
}>();

const connectionsGetter = useConnections();
const connections = computed(
  () => connectionsGetter(props.component.id, props.connections).value,
);
const incoming = computed(() => connections.value.incoming);
const outgoing = computed(() => connections.value.outgoing);
</script>
