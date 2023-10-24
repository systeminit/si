<template>
  <div v-if="componentId">
    <template
      v-if="fetchDebugReqStatus.isPending || !fetchDebugReqStatus.isRequested"
    >
      <LoadingMessage>Loading debug details...</LoadingMessage>
    </template>
    <template v-else-if="fetchDebugReqStatus.isError">
      <ErrorMessage :requestStatus="fetchDebugReqStatus" />
    </template>
    <template v-else-if="fetchDebugReqStatus.isSuccess && debugData">
      <div class="border border-neutral-500 m-xs">
        <Collapsible label="Component" defaultOpen textSize="lg">
          <dl class="border-l-2 p-2">
            <dt class="uppercase text-xs italic opacity-80">Id</dt>
            <dd class="p-2 my-2 border-2 border-opacity-10">
              <pre>{{ componentId }}</pre>
            </dd>
            <dt class="uppercase text-xs italic opacity-80">Variant Id</dt>
            <dd class="p-2 my-2 border-2 border-opacity-10">
              <pre>{{ debugData.schemaVariantId }}</pre>
            </dd>
          </dl>
        </Collapsible>
        <Collapsible
          label="Attributes"
          :defaultOpen="false"
          contentAs="ul"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.attributes"
            :key="attribute.path"
            class="m-2"
            :label="attribute.path"
            :defaultOpen="false"
            as="li"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>
        <Collapsible
          label="Input Sockets"
          :defaultOpen="false"
          contentAs="ul"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.inputSockets"
            :key="attribute.name"
            class="m-2"
            :label="attribute.name"
            :defaultOpen="false"
            as="li"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>
        <Collapsible
          label="Output Sockets"
          :defaultOpen="false"
          contentAs="ul"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.outputSockets"
            :key="attribute.name"
            class="m-2"
            :label="attribute.name"
            :defaultOpen="false"
            as="li"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import {
  Collapsible,
  ErrorMessage,
  LoadingMessage,
} from "@si/vue-lib/design-system";
import { PropType, computed, onMounted } from "vue";
import { ComponentId, useComponentsStore } from "@/store/components.store";
import AttributeDebugView from "./AttributeDebugView.vue";

const componentsStore = useComponentsStore();

const debugData = computed(
  () => componentsStore.debugDataByComponentId[props.componentId],
);

const props = defineProps({
  componentId: { type: String as PropType<ComponentId>, required: true },
});

const fetchDebugReqStatus = componentsStore.getRequestStatus(
  "FETCH_COMPONENT_DEBUG_VIEW",
  computed(() => props.componentId),
);

onMounted(() => {
  componentsStore.FETCH_COMPONENT_DEBUG_VIEW(props.componentId);
});
</script>
