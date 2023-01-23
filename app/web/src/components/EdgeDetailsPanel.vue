<template>
  <div class="flex flex-col h-full">
    <div class="p-xs border-b dark:border-neutral-600">
      <Inline align-y="center">
        <Icon size="md" name="plug" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Connection Details</div>
      </Inline>
    </div>

    <Stack spacing="none" class="p-xs border-b dark:border-neutral-600">
      <!-- <div class="ml-lg relative z-10">
        <div class="bg-shade-0 rounded-full w-sm h-sm -mt-sm" />
        <div class="bg-shade-0 w-[2px] h-md ml-[7px]" />
        <div class="bg-shade-0 rounded-full w-sm h-sm -mb-sm" />
      </div> -->
      <ComponentCard :component-id="fromComponent.id" />
      <div class="_connection-label text-xs italic">
        <!-- currently output and input socket always have the same label/name -->
        <span class="capsize">{{ fromSocket?.name }}</span>
        <!-- <div>to</div>
        <span class="capsize">{{ toSocket?.name }}</span> -->
      </div>
      <ComponentCard :component-id="toComponent.id" />
    </Stack>

    <Stack class="p-sm border-b dark:border-neutral-600" spacing="sm">
      <Inline
        v-if="changeStatus && changeStatus !== 'unmodified'"
        align-y="center"
      >
        <template v-if="changeStatus === 'added'">
          <Icon name="plus-circle" class="text-success-500" />
          <div class="capsize">Edge is new!</div>
        </template>
        <template v-else-if="changeStatus === 'modified'">
          <Icon name="tilde-circle" class="text-warning-500" />
          <div>Edge has been modified</div>
        </template>
        <template v-else-if="changeStatus === 'deleted'">
          <Icon name="minus-circle" class="text-destructive-500" />
          <div>Edge has been deleted</div>
        </template>
      </Inline>

      <div class="text-xs italic text-neutral-400 capsize">
        Created
        <Timestamp :date="selectedEdge.createdAt" size="long" />
        by Theo
      </div>
      <div
        v-if="changeStatus === 'deleted'"
        class="text-xs italic text-destructive-500 capsize"
      >
        Deleted
        <Timestamp :date="new Date(new Date())" size="long" />
        by Theo
      </div>
    </Stack>

    <!-- TODO: might want to connect these events to a global event bus (or the store?) instead of emitting? -->
    <div class="p-xs">
      <VButton2
        v-if="selectedEdge.changeStatus === 'deleted'"
        tone="destructive"
        variant="ghost"
        size="sm"
        icon="trash-restore"
        label="Restore"
        @click="emit('restore')"
      />
      <VButton2
        v-if="selectedEdge.changeStatus !== 'deleted'"
        tone="destructive"
        icon="trash"
        label="Delete"
        @click="emit('delete')"
      />
    </div>
  </div>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { computed } from "vue";
import { useComponentsStore } from "@/store/components.store";
import Timestamp from "@/ui-lib/Timestamp.vue";
import Icon from "@/ui-lib/icons/Icon.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Inline from "@/ui-lib/layout/Inline.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import ComponentCard from "./ComponentCard.vue";

const emit = defineEmits(["delete", "restore"]);

const componentsStore = useComponentsStore();

const selectedEdge = computed(() => componentsStore.selectedEdge);

const changeStatus = computed(() => selectedEdge.value.changeStatus);

const fromComponent = computed(
  () => componentsStore.componentsByNodeId[selectedEdge.value.fromNodeId],
);
const fromSchema = computed(
  () => componentsStore.schemaVariantsById[fromComponent.value.schemaVariantId],
);
const fromSocket = computed(() =>
  _.find(
    fromSchema.value.outputSockets,
    (s) => s.id === selectedEdge.value?.fromSocketId,
  ),
);
const toComponent = computed(
  () => componentsStore.componentsByNodeId[selectedEdge.value?.toNodeId],
);
// const toSchema = computed(
//   () => componentsStore.schemaVariantsById[toComponent.value.schemaVariantId],
// );
// const toSocket = computed(() =>
//   _.find(
//     toSchema.value.inputSockets,
//     (s) => s.id === selectedEdge.value?.toSocketId,
//   ),
// );
</script>

<style lang="less">
@socket-size: 12px;
._connection-label {
  border-left: 2px solid currentColor;
  padding: 15px;
  position: relative;
  z-index: 1;
  margin-left: 40px;

  &:before,
  &:after {
    content: "";
    width: @socket-size;
    height: @socket-size;
    border-radius: 100%;
    display: block;
    background: currentColor;
    position: absolute;
    margin-left: (-@socket-size / 2 - 1);
    left: 0;
  }
  &::before {
    top: 0;
    margin-top: -(@socket-size / 2);
  }
  &::after {
    margin-bottom: -(@socket-size / 2);
    bottom: 0;
  }
}
</style>
