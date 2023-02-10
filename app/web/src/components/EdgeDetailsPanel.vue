<template>
  <div class="flex flex-col h-full">
    <div class="p-xs border-b dark:border-neutral-600">
      <Inline align-y="center">
        <Icon size="md" name="plug" class="shrink-0 mr-2xs" />
        <div class="font-bold capsize">Connection Details</div>
      </Inline>
    </div>

    <div class="border-b dark:border-neutral-600">
      <div v-if="DEV_MODE" class="px-xs pt-xs text-2xs italic opacity-30">
        EDGE ID = {{ selectedEdge.id }}
      </div>

      <Stack spacing="none" class="p-xs">
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
      <DetailsPanelTimestamps
        :change-status="selectedEdge.changeStatus"
        :created="selectedEdge.createdInfo"
        :deleted="selectedEdge.deletedInfo"
      />
    </div>

    <template v-if="selectedEdge.changeStatus === 'deleted'">
      <Stack class="p-sm">
        <ErrorMessage icon="alert-triangle" tone="warning">
          This edge will be removed from your model when this change set is
          merged
        </ErrorMessage>
        <VButton2
          tone="shade"
          variant="ghost"
          size="md"
          icon="trash-restore"
          label="Restore edge"
          @click="emit('restore')"
        />
      </Stack>
    </template>
    <!-- <template v-else>
      <div class="p-sm">
        <VButton2
          tone="destructive"
          variant="ghost"
          icon="trash"
          label="Delete edge"
          @click="emit('delete')"
        />
      </div>
    </template> -->
  </div>
</template>

<script lang="ts" setup>
import _ from "lodash";
import { computed } from "vue";
import { useComponentsStore } from "@/store/components.store";
import Icon from "@/ui-lib/icons/Icon.vue";
import VButton2 from "@/ui-lib/VButton2.vue";
import Inline from "@/ui-lib/layout/Inline.vue";
import Stack from "@/ui-lib/layout/Stack.vue";
import ErrorMessage from "@/ui-lib/ErrorMessage.vue";
import ComponentCard from "./ComponentCard.vue";
import DetailsPanelTimestamps from "./DetailsPanelTimestamps.vue";

const emit = defineEmits(["delete", "restore"]);

const DEV_MODE = import.meta.env.DEV;

const componentsStore = useComponentsStore();

const selectedEdge = computed(() => componentsStore.selectedEdge);

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
