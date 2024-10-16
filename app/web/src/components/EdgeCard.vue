<template>
  <div v-if="fromComponent && toComponent">
    <ComponentCard :component="fromComponent" :titleCard="false" />
    <div class="_connection-label text-xs italic">
      <!-- currently output and input socket always have the same label/name -->
      <span class="capsize">{{ fromSocket?.name }}</span>
      <!-- <div>to</div>
        <span class="capsize">{{ toSocket?.name }}</span> -->
    </div>
    <ComponentCard :component="toComponent" :titleCard="false" />
  </div>
</template>

<script lang="ts" setup>
import * as _ from "lodash-es";
import { computed, PropType } from "vue";
import { useComponentsStore } from "@/store/components.store";
import { useAssetStore } from "@/store/asset.store";
import { EdgeId } from "@/api/sdf/dal/component";
import ComponentCard from "./ComponentCard.vue";

const props = defineProps({
  edgeId: { type: String as PropType<EdgeId>, required: true },
});

const componentsStore = useComponentsStore();
const assetStore = useAssetStore();

const edge = computed(() => componentsStore.rawEdgesById[props.edgeId]);

const fromComponent = computed(() =>
  edge.value?.fromComponentId
    ? componentsStore.allComponentsById[edge.value.fromComponentId]
    : undefined,
);
const fromSchemaVariant = computed(() =>
  fromComponent.value?.def.schemaVariantId
    ? assetStore.variantFromListById[fromComponent.value.def.schemaVariantId]
    : undefined,
);
const fromSocket = computed(() =>
  _.find(
    fromSchemaVariant.value?.outputSockets ?? [],
    (s) => s.id === edge.value?.fromSocketId,
  ),
);
const toComponent = computed(() =>
  edge.value?.toComponentId
    ? componentsStore.allComponentsById[edge.value.toComponentId]
    : undefined,
);
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
