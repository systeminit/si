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
        <!-- Component -->
        <Collapsible
          label="Component"
          defaultOpen
          textSize="lg"
          extraBorderAtBottomOfContent
        >
          <dl class="border-l-2 p-xs flex flex-col gap-xs">
            <DebugViewItem title="Id" :data="componentId" />
            <DebugViewItem
              title="Variant Id"
              :data="debugData.schemaVariantId"
            />
          </dl>
        </Collapsible>

        <!-- Attributes -->
        <Collapsible
          label="Attributes"
          :defaultOpen="false"
          contentAs="ul"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.attributes"
            :key="attribute.path"
            :label="attribute.path"
            :defaultOpen="false"
            as="li"
            xPadding="double"
            contentClasses="px-sm"
            extraBorderAtBottomOfContent
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>

        <!-- Input Sockets -->
        <Collapsible
          label="Input Sockets"
          :defaultOpen="false"
          contentAs="ul"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.inputSockets"
            :key="attribute.name"
            xPadding="double"
            contentClasses="px-sm"
            extraBorderAtBottomOfContent
            :label="attribute.name"
            :defaultOpen="false"
            as="li"
          >
            <AttributeDebugView :data="attribute.debugData" />
          </Collapsible>
        </Collapsible>

        <!-- Output Sockets -->
        <Collapsible
          label="Output Sockets"
          :defaultOpen="false"
          contentAs="ul"
          textSize="lg"
        >
          <Collapsible
            v-for="attribute in debugData.outputSockets"
            :key="attribute.name"
            xPadding="double"
            contentClasses="px-sm"
            extraBorderAtBottomOfContent
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
import DebugViewItem from "./DebugViewItem.vue";

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
