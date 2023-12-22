<template>
  <div v-if="fromComponent && toComponent">
    <ComponentCard :componentId="fromComponent.id" />
    <div class="_connection-label text-xs italic">
      <!-- currently output and input socket always have the same label/name -->
      <span class="capsize">{{ fromSocket?.name }}</span>
      <!-- <div>to</div>
        <span class="capsize">{{ toSocket?.name }}</span> -->
    </div>
    <ComponentCard :componentId="toComponent.id" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import { EdgeId, useComponentsStore } from "@/store/components.store";
import ComponentCard from "./ComponentCard.vue";

const props = defineProps({
  edgeId: { type: String as PropType<EdgeId>, required: true },
});

const componentsStore = useComponentsStore();

const edge = computed(() => componentsStore.edgesById[props.edgeId]);

const fromComponent = computed(() =>
  edge.value?.fromComponentId
    ? componentsStore.componentsById[edge.value.fromComponentId]
    : undefined,
);
const fromSchema = computed(() =>
  fromComponent.value?.schemaVariantId
    ? componentsStore.schemaVariantsById[fromComponent.value.schemaVariantId]
    : undefined,
);
const fromSocket = computed(() =>
  _.find(
    fromSchema.value?.outputSockets ?? [],
    (s) => s.id === edge.value?.fromExternalProviderId,
  ),
);
const toComponent = computed(() =>
  edge.value?.toComponentId
    ? componentsStore.componentsById[edge.value.toComponentId]
    : undefined,
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
